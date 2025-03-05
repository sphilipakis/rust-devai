use crate::packer::PackToml;
use crate::packer::pack_toml::{PartialPackToml, parse_validate_pack_toml};
use crate::support::zip;
use crate::{Error, Result};
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
}

// endregion: --- Tests
