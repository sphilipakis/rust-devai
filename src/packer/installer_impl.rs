use crate::dir_context::DirContext;
use crate::pack::PackIdentity;
use crate::packer::PackToml;
use crate::packer::pack_toml::parse_validate_pack_toml;
use crate::packer::support;
use crate::support::zip;
use crate::{Error, Result};
use reqwest::Client;
use serde::Deserialize;
use simple_fs::{SPath, ensure_dir};
use std::str::FromStr;
use time::OffsetDateTime;
use time_tz::OffsetDateTimeExt;

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
			PackUri::RepoPack(identity) => write!(f, "{}", identity),
			PackUri::LocalPath(path) => write!(f, "local file '{}'", path),
			PackUri::HttpLink(url) => write!(f, "URL '{}'", url),
		}
	}
}

// endregion: --- PackUri

// region:    --- LatestToml

#[derive(Deserialize, Debug)]
struct LatestToml {
	latest_stable: Option<LatestStableInfo>,
}

#[derive(Deserialize, Debug)]
struct LatestStableInfo {
	version: Option<String>,
	rel_path: Option<String>,
}

impl LatestToml {
	fn validate(&self) -> Result<(&str, &str)> {
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

pub struct InstalledPack {
	pub pack_toml: PackToml,
	pub path: SPath,
	#[allow(unused)]
	pub size: usize,
	pub zip_size: usize,
}

/// Install a `file.aipack` into the .aipack-base/pack/installed directory
///
/// IMPORTANT: Right now, very prelimealy. Should do the following:
///
/// TODO:
/// - Check for an existing installed pack.
/// - If an already installed pack has a semver greater than the new one,
///   return an error so that the caller can handle it with a prompt, and then provide a force flag, for example.
/// - Probably need to remove the existing pack files; otherwise, some leftover files can be an issue.
///
/// Returns the InstalledPack with information about the installed pack.
pub async fn install_pack(dir_context: &DirContext, pack_uri: &str) -> Result<InstalledPack> {
	let pack_uri = PackUri::parse(pack_uri);

	// Get the aipack file path, downloading if needed
	let (aipack_zipped_file, pack_uri) = match pack_uri {
		pack_uri @ PackUri::RepoPack(_) => download_from_repo(dir_context, pack_uri).await?,
		pack_uri @ PackUri::LocalPath(_) => resolve_local_path(dir_context, pack_uri)?,
		pack_uri @ PackUri::HttpLink(_) => download_pack(dir_context, pack_uri).await?,
	};

	// Validate file exists and has correct extension
	support::validate_aipack_file(&aipack_zipped_file, &pack_uri.to_string())?;

	// Get the zip file size
	let zip_size = support::get_file_size(&aipack_zipped_file, &pack_uri.to_string())?;

	// Common installation steps for both local and remote files
	let mut installed_pack = install_aipack_file(dir_context, &aipack_zipped_file, &pack_uri)?;
	installed_pack.zip_size = zip_size;

	Ok(installed_pack)
}

/// Downloads a pack from the repository based on PackIdentity
async fn download_from_repo(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	if let PackUri::RepoPack(ref pack_identity) = pack_uri {
		// Construct the URL to the latest.toml file
		let latest_toml_url = format!(
			"https://repo.aipack.ai/pack/{}/{}/stable/latest.toml",
			pack_identity.namespace, pack_identity.name
		);

		// Fetch the latest.toml file
		let client = Client::new();
		let response = client.get(&latest_toml_url).send().await.map_err(|e| Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!("Failed to download latest.toml: {}", e),
		})?;

		// Check if the request was successful
		if !response.status().is_success() {
			return Err(Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause: format!("HTTP error when fetching latest.toml: {}", response.status()),
			});
		}

		// Parse the latest.toml content
		let latest_toml_content = response.text().await.map_err(|e| Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!("Failed to read latest.toml content: {}", e),
		})?;

		let latest_toml: LatestToml = toml::from_str(&latest_toml_content).map_err(|e| Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!("Failed to parse latest.toml: {}", e),
		})?;

		// Validate the latest.toml content
		let (_version, rel_path) = latest_toml.validate()?;

		// Construct the full URL to the .aipack file
		let base_url = format!(
			"https://repo.aipack.ai/pack/{}/{}/stable/",
			pack_identity.namespace, pack_identity.name
		);

		let aipack_url = format!("{}{}", base_url, rel_path);

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
fn resolve_local_path(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	if let PackUri::LocalPath(ref path) = pack_uri {
		let aipack_zipped_file = SPath::from(path);

		if aipack_zipped_file.path().is_absolute() {
			Ok((aipack_zipped_file, pack_uri))
		} else {
			let absolute_path = dir_context.current_dir().join_str(aipack_zipped_file.to_str());
			Ok((absolute_path, pack_uri))
		}
	} else {
		Err(Error::custom(
			"Expected LocalPath variant but got a different one".to_string(),
		))
	}
}

/// Downloads a pack from a URL and returns the path to the downloaded file
async fn download_pack(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	if let PackUri::HttpLink(ref url) = pack_uri {
		// Get the download directory
		let download_dir = dir_context.aipack_paths().get_base_pack_download_dir()?;

		// Create the download directory if it doesn't exist
		if !download_dir.exists() {
			ensure_dir(&download_dir)?;
		}

		// Extract the filename from the URL
		let url_path = url.split('/').last().unwrap_or("unknown.aipack");
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
					cause: format!("Failed to format timestamp: {}", e),
				})?;

		// Create a cleaner timestamp for filenames (removing colons, etc.)
		let file_timestamp = timestamp.replace([':', 'T'], "-");
		let file_timestamp = file_timestamp.split('.').next().unwrap_or(timestamp.as_str());
		let timestamped_filename = format!("{}-{}", file_timestamp, filename);
		let download_path = download_dir.join_str(&timestamped_filename);

		// Download the file
		let client = Client::new();
		let response = client.get(url).send().await.map_err(|e| Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!("Failed to download file: {}", e),
		})?;

		// Check if the request was successful
		if !response.status().is_success() {
			return Err(Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause: format!("HTTP error: {}", response.status()),
			});
		}

		// Stream the response body to file
		let mut stream = response.bytes_stream();
		use tokio::fs::File as TokioFile;
		use tokio::io::AsyncWriteExt;

		// We need to use tokio's async file for proper streaming
		let mut file = TokioFile::create(download_path.path())
			.await
			.map_err(|e| Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause: format!("Failed to create file: {}", e),
			})?;

		while let Some(chunk_result) = tokio_stream::StreamExt::next(&mut stream).await {
			let chunk = chunk_result.map_err(|e| Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause: format!("Failed to download chunk: {}", e),
			})?;

			file.write_all(&chunk).await.map_err(|e| Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause: format!("Failed to write chunk to file: {}", e),
			})?;
		}

		file.flush().await.map_err(|e| Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!("Failed to flush file: {}", e),
		})?;

		return Ok((download_path, pack_uri));
	}

	Err(Error::custom(
		"Expected HttpLink variant but got a different one".to_string(),
	))
}

/// Common installation logic for both local and remote aipack files
/// Return the InstalledPack containing pack information and installation details
fn install_aipack_file(
	dir_context: &DirContext,
	aipack_zipped_file: &SPath,
	pack_uri: &PackUri,
) -> Result<InstalledPack> {
	// -- Get the aipack base pack install dir
	// This is the pack base dir and now, we need ot add `namespace/pack_name`
	let pack_installed_dir = dir_context.aipack_paths().get_base_pack_installed_dir()?;

	// Now, we automatically create, so we do not require it to be init-base
	ensure_dir(&pack_installed_dir)?;

	// Note: This should not happen, as it should have failed in the ensure_dir above.
	//       Howeer, for now,
	if !pack_installed_dir.exists() {
		return Err(Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!(
				"aipack base directory '{pack_installed_dir}' not found.\n   recommendation: Run 'aip init'"
			),
		});
	}

	// -- Extract the pack.toml from zip and validate
	let new_pack_toml = support::extract_pack_toml_from_pack_file(aipack_zipped_file)?;

	// NEW: Validate prerelease format for installation
	support::validate_version_for_install(&new_pack_toml.version)?;

	// -- Check if a pack with the same namespace/name is already installed
	let potential_existing_path = pack_installed_dir
		.join_str(&new_pack_toml.namespace)
		.join_str(&new_pack_toml.name);

	if potential_existing_path.exists() {
		// If an existing pack is found, we need to check its version
		let existing_pack_toml_path = potential_existing_path.join_str("pack.toml");

		if existing_pack_toml_path.exists() {
			// Read the existing pack.toml file
			let existing_toml_content =
				std::fs::read_to_string(existing_pack_toml_path.path()).map_err(|e| Error::FailToInstall {
					aipack_ref: pack_uri.to_string(),
					cause: format!("Failed to read existing pack.toml: {}", e),
				})?;

			// Parse the existing pack.toml
			let existing_pack_toml =
				parse_validate_pack_toml(&existing_toml_content, existing_pack_toml_path.to_str())?;

			// Check if the installed version is greater than the new version
			support::validate_version_update(&existing_pack_toml.version, &new_pack_toml.version)?;
		}
	}

	// If we've gotten here, either there's no existing pack or the new version is greater than or equal to the installed version
	let pack_target_dir = pack_installed_dir
		.join_str(&new_pack_toml.namespace)
		.join_str(&new_pack_toml.name);

	// If the directory exists, remove it first to ensure clean installation
	if pack_target_dir.exists() {
		std::fs::remove_dir_all(pack_target_dir.path()).map_err(|e| Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!("Failed to remove existing pack directory: {}", e),
		})?;
	}

	zip::unzip_file(aipack_zipped_file, &pack_target_dir).map_err(|e| Error::FailToInstall {
		aipack_ref: pack_uri.to_string(),
		cause: format!("Failed to unzip pack: {}", e),
	})?;

	// Calculate the size of the installed pack
	let size = support::calculate_directory_size(&pack_target_dir)?;

	Ok(InstalledPack {
		pack_toml: new_pack_toml,
		path: pack_target_dir,
		size,
		zip_size: 0, // This will be populated by the caller
	})
}

// region:    --- Tests

#[cfg(test)]
#[path = "../_tests/tests_installer_impl.rs"]
mod tests_installer_impl;

// endregion: --- Tests
