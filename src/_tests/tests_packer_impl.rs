use crate::_test_support::remove_test_dir;
use crate::exec::packer::{self};
use crate::runtime::Runtime;
use simple_fs::SPath;
use std::fs;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_packer_impl_pack_simple() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir()?;
	let dir_context = runtime.dir_context();
	let to_pack_dir = SPath::new("tests-data/test_packs_folder/test_pack_01");

	// -- Exec
	let pack_result = packer::pack_dir(to_pack_dir, dir_context.current_dir())?;

	// -- Check
	// Verify that the pack file was created with correct structure
	verify_aipack_file(&pack_result.pack_file)?;

	// Verify pack information is correct
	assert_eq!(pack_result.pack_toml.namespace, "test");
	assert_eq!(pack_result.pack_toml.name, "test_pack_01");
	assert_eq!(pack_result.pack_toml.version, "0.1.0");

	// Verify the filename follows the expected pattern
	let filename = pack_result.pack_file.name();
	assert!(
		filename.starts_with("test@test_pack_01-v0.1.0"),
		"Unexpected filename: {}",
		filename
	);

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

// region:    --- Support

// Test helper to verify the structure of a created .aipack file
fn verify_aipack_file(aipack_path: &SPath) -> Result<()> {
	// Check that the file exists
	assert!(
		aipack_path.exists(),
		"The .aipack file was not created at {}",
		aipack_path
	);

	// Check that it has the correct extension
	assert_eq!(aipack_path.ext(), "aipack", "The file does not have .aipack extension");

	// Check that the file size is reasonable (greater than a minimal size)
	let metadata = fs::metadata(aipack_path.path())?;
	assert!(
		metadata.len() > 100,
		"The .aipack file is too small: {} bytes",
		metadata.len()
	);

	Ok(())
}

// endregion: --- Support
