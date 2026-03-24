use crate::exec::packer::PackToml;
use crate::exec::packer::pack_toml::{PartialPackToml, parse_validate_pack_toml};
use crate::support::zip;
use crate::{Error, Result};
use lazy_regex::regex;
use reqwest::Client;
use semver::Version;
use simple_fs::SPath;
use simple_fs::ensure_dir;
use time::OffsetDateTime;
use time_tz::OffsetDateTimeExt;

use crate::dir_context::DirContext;
use crate::support::webc;
use crate::types::PackIdentity;

use serde::Deserialize;
use std::str::FromStr;

// region:    --- PackUri

#[derive(Debug, Clone)]
pub enum PackUri {
	RepoPack(PackIdentity),
	LocalPath(String),
	HttpLink(String),
}

impl PackUri {
	pub fn parse(uri: &str) -> Self {
		// Try to parse as PackIdentity first
		if let Ok(pack_identity) = PackIdentity::from_str(uri) {
			return PackUri::RepoPack(pack_identity);
		}

		// If not a PackIdentity, check if it's an HTTP link
		if uri.starts_with("http://") || uri.starts_with("https://") {
			PackUri::HttpLink(uri.to_string())
		} else {
			// Otherwise, treat as local path
			PackUri::LocalPath(uri.to_string())
		}
	}
}

impl std::fmt::Display for PackUri {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PackUri::RepoPack(identity) => write!(f, "{identity}"),
			PackUri::LocalPath(path) => write!(f, "local file '{path}'"),
			PackUri::HttpLink(url) => write!(f, "URL '{url}'"),
		}
	}
}

// endregion: --- PackUri

// region:    --- LatestToml

#[derive(Deserialize, Debug)]
pub(super) struct LatestToml {
	pub latest_stable: Option<LatestStableInfo>,
}

#[derive(Deserialize, Debug)]
pub(super) struct LatestStableInfo {
	pub version: Option<String>,
	pub rel_path: Option<String>,
}

impl LatestToml {
	pub fn validate(&self) -> Result<(&str, &str)> {
		// Check if latest_stable exists
		let latest_stable = self
			.latest_stable
			.as_ref()
			.ok_or_else(|| Error::custom("Missing 'latest_stable' section in latest.toml".to_string()))?;

		// Check if version is provided
		let version = latest_stable
			.version
			.as_deref()
			.ok_or_else(|| Error::custom("Missing 'version' in latest_stable section of latest.toml".to_string()))?;

		// Check if rel_path is provided
		let rel_path = latest_stable
			.rel_path
			.as_deref()
			.ok_or_else(|| Error::custom("Missing 'rel_path' in latest_stable section of latest.toml".to_string()))?;

		Ok((version, rel_path))
	}
}

// endregion: --- LatestToml

// region:    --- Shared Repo/Download Helpers

/// Fetches the latest.toml metadata from the remote repository for a given pack identity.
///
/// Returns the parsed LatestToml, which can be validated for version and rel_path.
pub(super) async fn fetch_repo_latest_toml(pack_identity: &PackIdentity) -> Result<LatestToml> {
	let latest_toml_url = format!(
		"https://repo.aipack.ai/pack/{}/{}/stable/latest.toml",
		pack_identity.namespace, pack_identity.name
	);

	let client = Client::new();
	let response = client.get(&latest_toml_url).send().await.map_err(|e| Error::FailToInstall {
		aipack_ref: pack_identity.to_string(),
		cause: format!("Failed to download latest.toml: {e}"),
	})?;

	if !response.status().is_success() {
		return Err(Error::FailToInstall {
			aipack_ref: pack_identity.to_string(),
			cause: format!("HTTP error when fetching latest.toml: {}", response.status()),
		});
	}

	let latest_toml_content = response.text().await.map_err(|e| Error::FailToInstall {
		aipack_ref: pack_identity.to_string(),
		cause: format!("Failed to read latest.toml content: {e}"),
	})?;

	let latest_toml: LatestToml = toml::from_str(&latest_toml_content).map_err(|e| Error::FailToInstall {
		aipack_ref: pack_identity.to_string(),
		cause: format!("Failed to parse latest.toml: {e}"),
	})?;

	Ok(latest_toml)
}

/// Constructs the full download URL for a pack from its identity and a relative path from latest.toml.
pub(super) fn build_repo_pack_url(pack_identity: &PackIdentity, rel_path: &str) -> String {
	format!(
		"https://repo.aipack.ai/pack/{}/{}/stable/{rel_path}",
		pack_identity.namespace, pack_identity.name
	)
}

/// Downloads a pack from a repo pack identity, resolving via latest.toml.
///
/// Returns the path to the downloaded `.aipack` file and the original PackUri.
pub(super) async fn download_from_repo(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	if let PackUri::RepoPack(ref pack_identity) = pack_uri {
		let latest_toml = fetch_repo_latest_toml(pack_identity).await?;

		// Validate the latest.toml content
		let (_version, rel_path) = latest_toml.validate()?;

		// Construct the full URL to the .aipack file
		let aipack_url = build_repo_pack_url(pack_identity, rel_path);

		// Use HttpLink to download the actual pack
		let http_uri = PackUri::HttpLink(aipack_url);
		let (aipack_file, _) = download_pack(dir_context, http_uri).await?;

		return Ok((aipack_file, pack_uri));
	}

	Err(Error::custom(
		"Expected RepoPack variant but got a different one".to_string(),
	))
}

/// Resolves a local path to an absolute SPath
pub(super) fn resolve_local_path(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	if let PackUri::LocalPath(ref path) = pack_uri {
		let aipack_zipped_file = SPath::from(path);

		if aipack_zipped_file.path().is_absolute() {
			Ok((aipack_zipped_file, pack_uri))
		} else {
			let absolute_path = dir_context.current_dir().join(aipack_zipped_file.as_str());
			Ok((absolute_path, pack_uri))
		}
	} else {
		Err(Error::custom(
			"Expected LocalPath variant but got a different one".to_string(),
		))
	}
}

/// Downloads a pack from a URL and returns the path to the downloaded file
pub(super) async fn download_pack(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	if let PackUri::HttpLink(ref url) = pack_uri {
		// Get the download directory
		let download_dir = dir_context.aipack_paths().get_base_pack_download_dir()?;

		// Create the download directory if it doesn't exist
		if !download_dir.exists() {
			ensure_dir(&download_dir)?;
		}

		// Extract the filename from the URL
		let url_path = url.split('/').next_back().unwrap_or("unknown.aipack");
		let filename = url_path.replace(' ', "-");

		// Create a timestamped filename using the time crate
		let now = OffsetDateTime::now_utc();
		// attempt to get local now (otherwise, no big deal, same machine so should be consistent return)
		let now = if let Ok(local) = time_tz::system::get_timezone() {
			now.to_timezone(local)
		} else {
			now
		};

		let timestamp =
			now.format(&time::format_description::well_known::Rfc3339)
				.map_err(|e| Error::FailToInstall {
					aipack_ref: pack_uri.to_string(),
					cause: format!("Failed to format timestamp: {e}"),
				})?;

		// Create a cleaner timestamp for filenames (removing colons, etc.)
		let file_timestamp = timestamp.replace([':', 'T'], "-");
		let file_timestamp = file_timestamp.split('.').next().unwrap_or(timestamp.as_str());
		let timestamped_filename = format!("{file_timestamp}-{filename}");
		let download_path = download_dir.join(&timestamped_filename);

		// Download the file
		webc::web_download_to_file(url, &download_path).await?;

		return Ok((download_path, pack_uri));
	}

	Err(Error::custom(
		"Expected HttpLink variant but got a different one".to_string(),
	))
}

/// Fetches the latest remote version string from the repository for a given pack identity.
///
/// Returns `Ok(Some(version))` if the remote latest.toml was successfully fetched and validated,
/// or `Ok(None)` if the remote could not be reached or the metadata was invalid.
pub(super) async fn fetch_repo_latest_version(pack_identity: &PackIdentity) -> Result<Option<String>> {
	match fetch_repo_latest_toml(pack_identity).await {
		Ok(latest_toml) => match latest_toml.validate() {
			Ok((version, _rel_path)) => Ok(Some(version.to_string())),
			Err(_) => Ok(None),
		},
		Err(_) => Ok(None),
	}
}

// endregion: --- Shared Repo/Download Helpers

/// Extracts and validates the pack.toml from an .aipack file
///
/// # Parameters
/// - `path_to_aipack`: The path to the .aipack file
///
/// # Returns
/// - Ok(PackToml): If extraction and validation are successful
/// - Err(Error): If any error occurs during extraction or validation
pub fn extract_pack_toml_from_pack_file(path_to_aipack: &SPath) -> Result<PackToml> {
	// Extract the pack.toml from zip
	let toml_content = zip::extract_text_content(path_to_aipack, "pack.toml").map_err(|e| Error::FailToInstall {
		aipack_ref: path_to_aipack.as_str().to_string(),
		cause: format!("Failed to extract pack.toml: {e}"),
	})?;

	// Parse and validate the pack.toml content
	let pack_toml =
		parse_validate_pack_toml(&toml_content, &format!("pack.toml for {path_to_aipack}")).map_err(|e| {
			Error::FailToInstall {
				aipack_ref: path_to_aipack.as_str().to_string(),
				cause: format!("Invalid pack.toml: {e}"),
			}
		})?;

	Ok(pack_toml)
}

/// Extracts the pack.toml from an .aipack file and returns it as a PartialPackToml without validation
///
/// This function is useful when custom error handling is needed or when only checking
/// specific fields without full validation.
///
/// # Parameters
/// - `path_to_aipack`: The path to the .aipack file
///
/// # Returns
/// - Ok(PartialPackToml): If extraction is successful
/// - Err(Error): If any error occurs during extraction
#[allow(unused)]
pub fn extract_partial_pack_toml_from_pack_file(path_to_aipack: &SPath) -> Result<PartialPackToml> {
	// Extract the pack.toml from zip
	let toml_content = zip::extract_text_content(path_to_aipack, "pack.toml").map_err(|e| Error::FailToInstall {
		aipack_ref: path_to_aipack.as_str().to_string(),
		cause: format!("Failed to extract pack.toml: {e}"),
	})?;

	// Parse the TOML content without validation
	let partial_pack_toml = toml::from_str(&toml_content).map_err(|e| Error::FailToInstall {
		aipack_ref: path_to_aipack.as_str().to_string(),
		cause: format!("Failed to parse pack.toml: {e}"),
	})?;

	Ok(partial_pack_toml)
}

/// Validates an .aipack file extension and existence
///
/// # Parameters
/// - `aipack_file`: The path to the .aipack file
/// - `reference`: A string representation of the file for error reporting
///
/// # Returns
/// - Ok(()): If validation passes
/// - Err(Error): If validation fails
pub fn validate_aipack_file(aipack_file: &SPath, reference: &str) -> Result<()> {
	if !aipack_file.exists() {
		return Err(Error::FailToInstall {
			aipack_ref: reference.to_string(),
			cause: "aipack file does not exist".to_string(),
		});
	}

	if aipack_file.ext() != "aipack" {
		return Err(Error::FailToInstall {
			aipack_ref: reference.to_string(),
			cause: format!("aipack file must be '.aipack' file, but was {}", aipack_file.name()),
		});
	}

	Ok(())
}

/// Validates if the new version is greater than or equal to the installed version
///
/// Returns Ok(()) if the new version is greater than or equal to the installed version
/// or if either version can't be parsed as a valid semver version.
///
/// Returns Err(Error::InstallFailInstalledVersionAbove) if the installed version is greater
/// than the new version.
///
/// # Parameters
/// - `installed_version`: The currently installed version
/// - `new_version`: The new version to be installed
///
/// # Returns
/// - Ok(()): If version comparison passes
/// - Err(Error): If validation fails
pub fn validate_version_update(installed_version: &str, new_version: &str) -> Result<std::cmp::Ordering> {
	// Remove leading 'v' if present for both versions
	let installed = installed_version.trim_start_matches('v');
	let new = new_version.trim_start_matches('v');

	// Parse versions into semver::Version
	if let (Ok(installed_semver), Ok(new_semver)) = (Version::parse(installed), Version::parse(new)) {
		Ok(new_semver.cmp(&installed_semver))
	} else {
		// If not valid semver, fallback to string comparison
		Ok(new.cmp(installed))
	}
}

/// Validates if the version format is valid for installation
///
/// In addition to standard semver validation, this function checks that
/// prerelease versions (e.g., -alpha, -beta) must end with a .number
///
/// Examples of valid versions:
/// - 0.1.1
/// - 0.1.1-alpha.1
/// - 0.1.1-beta.123
/// - 0.1.1-rc.1.2
///
/// Examples of invalid versions:
/// - 0.1.1-alpha (missing .number)
/// - 0.1.1-alpha.text (not ending with number)
///
/// # Parameters
/// - `version`: The version string to validate
///
/// # Returns
/// - Ok(()): If the version format is valid
/// - Err(Error): If the version format is invalid
pub fn validate_version_for_install(version: &str) -> Result<()> {
	// Remove leading 'v' if present
	let version_str = version.trim_start_matches('v');

	// Check if there's a prerelease portion (after a hyphen)
	if let Some(hyphen_idx) = version_str.find('-') {
		let prerelease = &version_str[hyphen_idx + 1..];

		// Regex to check if the prerelease ends with .number
		// This matches: any characters followed by a dot and then one or more digits at the end
		let prerelease_ending_with_number = regex!(r"\.[0-9]+$");

		if !prerelease_ending_with_number.is_match(prerelease) {
			return Err(Error::InvalidPrereleaseFormat {
				version: version.to_string(),
			});
		}
	}

	Ok(())
}

// /// Normalizes a version string by replacing dots and special characters with hyphens
// /// This is just to write the file names (cosmetic)
// /// and ensuring no consecutive hyphens
// pub fn normalize_version(version: &str) -> String {
// 	let mut result = String::new();
// 	let mut last_was_hyphen = false;

// 	for c in version.chars() {
// 		if c.is_alphanumeric() {
// 			result.push(c);
// 			last_was_hyphen = false;
// 		} else if !last_was_hyphen {
// 			result.push('-');
// 			last_was_hyphen = true;
// 		}
// 	}

// 	// Remove trailing hyphen if exists
// 	if result.ends_with('-') {
// 		result.pop();
// 	}

// 	result
// }

/// Get the size of a file in bytes
pub fn get_file_size(file_path: &SPath, reference: &str) -> Result<usize> {
	let metadata = std::fs::metadata(file_path.path()).map_err(|e| Error::FailToInstall {
		aipack_ref: reference.to_string(),
		cause: format!("Failed to get file metadata: {e}"),
	})?;

	Ok(metadata.len() as usize)
}

/// Calculate the total size of a directory recursively
pub fn calculate_directory_size(dir_path: &SPath) -> Result<usize> {
	use walkdir::WalkDir;

	let total_size = WalkDir::new(dir_path.path())
		.into_iter()
		.filter_map(|entry| entry.ok())
		.filter_map(|entry| entry.metadata().ok())
		.filter(|metadata| metadata.is_file())
		.map(|metadata| metadata.len() as usize)
		.sum();

	Ok(total_size)
}

// region:    --- Tests

#[cfg(test)]
#[path = "support_tests.rs"]
mod tests;

// endregion: --- Tests
