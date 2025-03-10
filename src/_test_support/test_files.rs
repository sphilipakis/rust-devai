use super::Result;
use simple_fs::SPath;
use std::fs;
use time::OffsetDateTime;

const TEST_TMP_DIR: &str = "./tests-data/.tmp";

/// Generate a unique directory name inside tests-data/.tmp/ using pseudo unique enough name
/// This is just the dir name, like `test-3412341234-323432`, no path
/// Use `gen_test_path(gen_test_dir_name())` to get the `property path`
pub fn gen_test_dir_path() -> SPath {
	// Suffi sufficient for test directories
	let now = OffsetDateTime::now_utc();
	let path = format!("test-{}-{}", now.unix_timestamp(), now.microsecond());

	gen_tmp_test_path(&path)
}

/// Resolve a path relative to tests-data/.tmp/ directory
pub fn gen_tmp_test_path(path: &str) -> SPath {
	SPath::new(TEST_TMP_DIR).join(path)
}

/// Saves the given content to the specified path.
/// Ensures that the parent directory exists before saving.
///
/// # Parameters
/// - `path`: The file path where the content should be saved.
/// - `content`: The content to write to the file.
///
/// # Returns
/// - Ok(()) if the file was saved successfully.
/// - Err(Error) if any IO error occurs.
pub fn save_file_content(path: &SPath, content: &str) -> Result<()> {
	if let Some(parent) = path.path().parent() {
		fs::create_dir_all(parent)?;
	}
	fs::write(path.path(), content)?;
	Ok(())
}

/// Create a test file in tests-data/.tmp/ directory
#[allow(unused)]
pub fn create_test_file(path: &str, content: &str) -> Result<SPath> {
	let file_path = gen_tmp_test_path(path);

	// Create parent directories if they don't exist
	if let Some(parent) = file_path.parent() {
		fs::create_dir_all(parent.path())?;
	}

	fs::write(file_path.path(), content)?;
	Ok(file_path)
}

/// Create a test directory in tests-data/.tmp/ directory
#[allow(unused)]
pub fn create_test_dir(path: &str) -> Result<SPath> {
	let dir_path = gen_tmp_test_path(path);
	fs::create_dir_all(dir_path.path())?;
	Ok(dir_path)
}

/// Safely remove a test file
#[allow(unused)]
pub fn remove_test_file(path: SPath) -> Result<()> {
	// Safety check: make sure the path contains tests-data
	ensure_test_tmp_dir_path_safe(&path)?;

	// If file exists, remove it
	if path.exists() {
		fs::remove_file(path.path())?;
	}

	Ok(())
}

/// Safely remove a test directory and all its contents
pub fn remove_test_dir(path: &SPath) -> Result<()> {
	// Safety check: make sure the path contains tests-data
	ensure_test_tmp_dir_path_safe(path)?;

	// If directory exists, remove it recursively
	if path.exists() {
		fs::remove_dir_all(path.path())?;
	}

	Ok(())
}

/// Ensure the path is within tests-data to prevent accidental deletion of important files
fn ensure_test_tmp_dir_path_safe(path: &SPath) -> Result<()> {
	// Get the canonical path to resolve any .. or symbolic links
	let canonical = path.canonicalize()?;

	// Check if the canonical path contains tests-data
	if !canonical.as_str().contains("tests-data/.tmp") {
		return Err(format!("Safety check failed: Path must be within tests-data directory: {canonical}").into());
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_test_support_gen_test_dir_path() {
		let name1 = gen_test_dir_path();
		let name2 = gen_test_dir_path();

		assert!(name1.as_str().starts_with("./tests-data/.tmp/test-"));
		assert!(name2.as_str().starts_with("./tests-data/.tmp/test-"));
		assert_ne!(name1.as_str(), name2.as_str(), "Generated names should be unique");
	}

	#[test]
	fn test_test_support_gen_tmp_test_path() {
		let path = gen_tmp_test_path("subdir/file.txt");
		assert!(path.as_str().contains("tests-data/.tmp/subdir/file.txt"));
	}
}
