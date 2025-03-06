use crate::packer::PackToml;
use crate::packer::pack_toml::{PartialPackToml, parse_validate_pack_toml};
use crate::support::zip;
use crate::{Error, Result};
use lazy_regex::regex;
use semver::Version;
use simple_fs::SPath;

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
		aipack_ref: path_to_aipack.to_str().to_string(),
		cause: format!("Failed to extract pack.toml: {}", e),
	})?;

	// Parse and validate the pack.toml content
	let pack_toml =
		parse_validate_pack_toml(&toml_content, &format!("pack.toml for {}", path_to_aipack)).map_err(|e| {
			Error::FailToInstall {
				aipack_ref: path_to_aipack.to_str().to_string(),
				cause: format!("Invalid pack.toml: {}", e),
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
		aipack_ref: path_to_aipack.to_str().to_string(),
		cause: format!("Failed to extract pack.toml: {}", e),
	})?;

	// Parse the TOML content without validation
	let partial_pack_toml = toml::from_str(&toml_content).map_err(|e| Error::FailToInstall {
		aipack_ref: path_to_aipack.to_str().to_string(),
		cause: format!("Failed to parse pack.toml: {}", e),
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
pub fn validate_version_update(installed_version: &str, new_version: &str) -> Result<()> {
	// Remove leading 'v' if present for both versions
	let installed = installed_version.trim_start_matches('v');
	let new = new_version.trim_start_matches('v');

	// Parse versions into semver::Version
	match (Version::parse(installed), Version::parse(new)) {
		(Ok(installed_semver), Ok(new_semver)) => {
			// Check if installed version is greater than new version
			if installed_semver > new_semver {
				return Err(Error::InstallFailInstalledVersionAbove {
					installed_version: installed_version.to_string(),
					new_version: new_version.to_string(),
				});
			}
		}
		// If we can't parse one of the versions, we'll just allow the installation
		// since we can't reliably determine which is newer
		_ => {}
	}

	Ok(())
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

/// Normalizes a version string by replacing dots and special characters with hyphens
/// This is just to write the file names (cosmetic)
/// and ensuring no consecutive hyphens
pub fn normalize_version(version: &str) -> String {
	let mut result = String::new();
	let mut last_was_hyphen = false;

	for c in version.chars() {
		if c.is_alphanumeric() {
			result.push(c);
			last_was_hyphen = false;
		} else if !last_was_hyphen {
			result.push('-');
			last_was_hyphen = true;
		}
	}

	// Remove trailing hyphen if exists
	if result.ends_with('-') {
		result.pop();
	}

	result
}

/// Get the size of a file in bytes
pub fn get_file_size(file_path: &SPath, reference: &str) -> Result<usize> {
	let metadata = std::fs::metadata(file_path.path()).map_err(|e| Error::FailToInstall {
		aipack_ref: reference.to_string(),
		cause: format!("Failed to get file metadata: {}", e),
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
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::Error;

	#[test]
	fn test_packer_support_normalize_version_simple() -> Result<()> {
		assert_eq!(normalize_version("1.0.0"), "1-0-0");
		assert_eq!(normalize_version("1.0-alpha"), "1-0-alpha");
		assert_eq!(normalize_version("1.0 beta"), "1-0-beta");
		assert_eq!(normalize_version("1.0-beta-2"), "1-0-beta-2");
		assert_eq!(normalize_version("1.0--beta--2"), "1-0-beta-2");
		assert_eq!(normalize_version("v1.0.0_rc1"), "v1-0-0-rc1");
		assert_eq!(normalize_version("1.0.0!@#$%^&*()"), "1-0-0");

		Ok(())
	}

	#[test]
	fn test_validate_version_update() -> Result<()> {
		// Test case: New version is greater than installed
		assert!(validate_version_update("1.0.0", "1.0.1").is_ok());
		assert!(validate_version_update("1.0.0", "1.1.0").is_ok());
		assert!(validate_version_update("1.0.0", "2.0.0").is_ok());

		// Test case: New version is equal to installed
		assert!(validate_version_update("1.0.0", "1.0.0").is_ok());

		// Test case: New version is less than installed
		let err = validate_version_update("1.0.1", "1.0.0").unwrap_err();
		match err {
			Error::InstallFailInstalledVersionAbove {
				installed_version,
				new_version,
			} => {
				assert_eq!(installed_version, "1.0.1");
				assert_eq!(new_version, "1.0.0");
			}
			_ => panic!("Expected InstallFailInstalledVersionAbove error"),
		}

		// Test with leading 'v'
		assert!(validate_version_update("v1.0.0", "1.0.1").is_ok());
		assert!(validate_version_update("1.0.0", "v1.0.1").is_ok());

		// Test with invalid versions (should pass since we can't compare them)
		assert!(validate_version_update("invalid", "1.0.0").is_ok());
		assert!(validate_version_update("1.0.0", "invalid").is_ok());
		assert!(validate_version_update("invalid", "invalid").is_ok());

		Ok(())
	}

	#[test]
	fn test_validate_version_for_install() -> Result<()> {
		// Test valid versions
		assert!(validate_version_for_install("0.1.0").is_ok());
		assert!(validate_version_for_install("1.0.0").is_ok());
		assert!(validate_version_for_install("0.1.1-alpha.1").is_ok());
		assert!(validate_version_for_install("0.1.1-beta.123").is_ok());
		assert!(validate_version_for_install("0.1.1-rc.1.2").is_ok());
		assert!(validate_version_for_install("v1.0.0-alpha.1").is_ok());

		// Test invalid versions
		let err = validate_version_for_install("0.1.1-alpha").unwrap_err();
		match err {
			Error::InvalidPrereleaseFormat { version } => {
				assert_eq!(version, "0.1.1-alpha");
			}
			_ => panic!("Expected InvalidPrereleaseFormat error"),
		}

		let err = validate_version_for_install("0.1.1-alpha.text").unwrap_err();
		match err {
			Error::InvalidPrereleaseFormat { version } => {
				assert_eq!(version, "0.1.1-alpha.text");
			}
			_ => panic!("Expected InvalidPrereleaseFormat error"),
		}

		let err = validate_version_for_install("0.1.1-alpha.1.some").unwrap_err();
		match err {
			Error::InvalidPrereleaseFormat { version } => {
				assert_eq!(version, "0.1.1-alpha.1.some");
			}
			_ => panic!("Expected InvalidPrereleaseFormat error"),
		}

		Ok(())
	}
}

// endregion: --- Tests
