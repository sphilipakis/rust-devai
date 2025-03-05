use super::*;
use crate::_test_support::{create_test_dir, remove_test_dir};
use crate::packer::{self, PackToml, install_pack};
use crate::run::Runtime;
use simple_fs::SPath;
use std::fs;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_installer_impl_local_file_simple() -> Result<()> {
	// -- Setup & Fixtures
	// this will create the .tests-data/.tmp/... and the base di for .aipack/ and .aipack-base
	let runtime = Runtime::new_test_runtime_for_temp_dir()?;
	let dir_context = runtime.dir_context();
	// Prep the pack dir
	let to_pack_dir = SPath::new("tests-data/test_packs_folder/test_pack_01")?;
	let pack_result = packer::pack_dir(to_pack_dir, dir_context.current_dir())?;
	let aipack_file_path = pack_result.pack_file;

	// -- Exec
	let installed_pack = install_pack(dir_context, aipack_file_path.to_str()).await?;

	// -- Check
	// Verify that the pack was installed correctly
	assert_eq!(installed_pack.pack_toml.namespace, "test");
	assert_eq!(installed_pack.pack_toml.name, "test_pack_01");
	assert_eq!(installed_pack.pack_toml.version, "0.1.0");

	// Verify the installation path follows the expected pattern
	let expected_install_path = dir_context
		.aipack_paths()
		.get_base_pack_installed_dir()?
		.join_str("test/test_pack_01");
	assert_eq!(installed_pack.path.to_str(), expected_install_path.to_str());

	// Verify that the main.aip file was extracted
	let main_aip_path = expected_install_path.join_str("main.aip");
	assert!(main_aip_path.exists(), "main.aip should have been extracted");

	// Verify pack.toml was extracted
	let pack_toml_path = expected_install_path.join_str("pack.toml");
	assert!(pack_toml_path.exists(), "pack.toml should have been extracted");

	// Verify zip_size is set correctly
	assert!(installed_pack.zip_size > 0, "zip_size should be greater than 0");

	// -- Cleanup
	// This will check that it is a `tests-data/.tmp`
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}
