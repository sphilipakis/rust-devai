use super::*;
use crate::_test_support::{remove_test_dir, save_file_content};
use crate::packer::{self, install_pack};
use crate::run::Runtime;
use simple_fs::{SPath, ensure_dir};

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_installer_impl_local_file_simple() -> Result<()> {
	// -- Setup & Fixtures
	// this will create the .tests-data/.tmp/... and the base dir for .aipack/ and .aipack-base
	let runtime = Runtime::new_test_runtime_for_temp_dir()?;
	let dir_context = runtime.dir_context();
	// Prep the pack dir
	let to_pack_dir = SPath::new("tests-data/test_packs_folder/test_pack_01");
	let pack_result = packer::pack_dir(to_pack_dir, dir_context.current_dir())?;
	let aipack_file_path = pack_result.pack_file;

	// -- Exec
	let installed_pack = install_pack(dir_context, aipack_file_path.as_str()).await?;

	// -- Check
	// Verify that the pack was installed correctly
	assert_eq!(installed_pack.pack_toml.namespace, "test");
	assert_eq!(installed_pack.pack_toml.name, "test_pack_01");
	assert_eq!(installed_pack.pack_toml.version, "0.1.0");

	// Verify the installation path follows the expected pattern
	let expected_install_path = dir_context
		.aipack_paths()
		.get_base_pack_installed_dir()?
		.join("test/test_pack_01");
	assert_eq!(installed_pack.path.as_str(), expected_install_path.as_str());

	// Verify that the main.aip file was extracted
	let main_aip_path = expected_install_path.join("main.aip");
	assert!(main_aip_path.exists(), "main.aip should have been extracted");

	// Verify pack.toml was extracted
	let pack_toml_path = expected_install_path.join("pack.toml");
	assert!(pack_toml_path.exists(), "pack.toml should have been extracted");

	// Verify zip_size is set correctly
	assert!(installed_pack.zip_size > 0, "zip_size should be greater than 0");

	// -- Cleanup
	// This will check that it is a `tests-data/.tmp`
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_local_version_above_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir()?;
	let dir_context = runtime.dir_context();

	// Create common main.aip content for both packs
	let main_aip_content = r#"# Test Main

This is a test agent file for installation testing."#;

	// Step 1: Create old pack directory (version 0.2.0)
	let old_pack_dir = dir_context.current_dir().join("pack_to_install/test_old/test-pack-01");
	ensure_dir(&old_pack_dir)?;
	let old_pack_toml = r#"
[pack]
namespace = "test_ns"
name = "test-pack-01"
version = "0.2.0"
"#;
	save_file_content(&old_pack_dir.join("pack.toml"), old_pack_toml)?;

	let old_pack_data = packer::pack_dir(&old_pack_dir, dir_context.current_dir())?;
	let old_pack_file = old_pack_data.pack_file;
	// Install the old pack (version 0.2.0)
	let _installed_old_pack = install_pack(dir_context, old_pack_file.as_str()).await?;
	// (Optional: assert that installed_old_pack.pack_toml.version == "0.2.0")

	// Step 2: Create new pack directory (version 0.1.0)
	let new_pack_dir = dir_context.current_dir().join("pack_to_install/test_new/test-pack-01");
	ensure_dir(&new_pack_dir)?;
	let new_pack_toml = r#"
[pack]
namespace = "test_ns"
name = "test-pack-01"
version = "0.1.0"
"#;
	save_file_content(&new_pack_dir.join("pack.toml"), new_pack_toml)?;
	save_file_content(&new_pack_dir.join("main.aip"), main_aip_content)?;
	let new_pack_data = packer::pack_dir(&new_pack_dir, dir_context.current_dir())?;
	let new_pack_file = new_pack_data.pack_file;

	// -- Execute: Try to install the new pack (version 0.1.0)
	let result = install_pack(dir_context, new_pack_file.as_str()).await;

	// -- Check: The new pack installation should fail.
	assert!(result.is_err(), "Installing lower version should fail");

	if let Err(error) = result {
		match error {
			Error::InstallFailInstalledVersionAbove {
				installed_version,
				new_version,
			} => {
				assert_eq!(installed_version, "0.2.0");
				assert_eq!(new_version, "0.1.0");
			}
			other => panic!("Expected InstallFailInstalledVersionAbove error, got: {:?}", other),
		}
	}

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;
	Ok(())
}

#[tokio::test]
async fn test_installer_impl_invalid_prerelease_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir()?;
	let dir_context = runtime.dir_context();

	// Create a pack directory with an invalid prerelease version "0.1.0-alpha"
	// (the prerelease must end with ".number" such as "-alpha.1")
	let invalid_pack_dir = dir_context
		.current_dir()
		.join("pack_to_install/invalid_prerelease/test-pack-02");
	ensure_dir(&invalid_pack_dir)?;
	let invalid_pack_toml = r#"
[pack]
namespace = "test_ns"
name = "test-pack-02"
version = "0.1.0-alpha"
"#;
	save_file_content(&invalid_pack_dir.join("pack.toml"), invalid_pack_toml)?;
	save_file_content(
		&invalid_pack_dir.join("main.aip"),
		"# Test Main\nInvalid prerelease version.",
	)?;

	// Pack the directory into a .aipack file
	let pack_data = packer::pack_dir(&invalid_pack_dir, dir_context.current_dir())?;
	let pack_file_str = pack_data.pack_file.as_str();

	// Attempt to install the pack, expecting an error due to invalid prerelease format
	let result = install_pack(dir_context, pack_file_str).await;

	assert!(
		result.is_err(),
		"Installation should fail due to invalid prerelease format"
	);

	if let Err(error) = result {
		match error {
			Error::InvalidPrereleaseFormat { version } => {
				assert_eq!(version, "0.1.0-alpha");
			}
			other => panic!("Expected InvalidPrereleaseFormat error, got: {:?}", other),
		}
	}

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;
	Ok(())
}
