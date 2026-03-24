//! Integration tests for unpack_pack

use crate::_test_support::{remove_test_dir, save_file_content};
use crate::exec::packer::unpack_pack;
use crate::runtime::Runtime;
use simple_fs::SPath;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

/// Test that unpack_pack fails when the destination already exists and --force is not set
#[tokio::test]
async fn test_unpacker_impl_unpack_dest_exists_no_force() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let dir_context = runtime.dir_context();

	// Compute and create the destination directory to simulate an existing unpack
	let aipack_wks_dir = dir_context
		.aipack_paths()
		.aipack_wks_dir()
		.ok_or("Should have aipack_wks_dir")?;
	let wks_custom_dir = aipack_wks_dir.get_pack_custom_dir()?;
	let dest_dir = wks_custom_dir.join("test_ns").join("test_pack");
	std::fs::create_dir_all(dest_dir.path())?;
	save_file_content(&dest_dir.join("marker.txt"), "existing")?;

	// -- Exec
	let result = unpack_pack(dir_context, "test_ns@test_pack", false).await;

	// -- Check
	assert!(result.is_err(), "Should fail when destination exists without --force");
	let err_msg = format!("{}", result.unwrap_err());
	assert!(
		err_msg.contains("already exists"),
		"Error should mention destination already exists, got: {err_msg}"
	);

	// -- Cleanup
	if dest_dir.exists() {
		std::fs::remove_dir_all(dest_dir.path())?;
	}

	Ok(())
}

/// Test that unpack_pack with --force replaces existing destination
/// Note: This test will attempt remote download which may fail in CI without network.
///       We still test the force-trash behavior by verifying the old content is removed.
#[tokio::test]
async fn test_unpacker_impl_unpack_dest_exists_with_force() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let dir_context = runtime.dir_context();

	// Compute and create the destination directory with a marker file
	let aipack_wks_dir = dir_context
		.aipack_paths()
		.aipack_wks_dir()
		.ok_or("Should have aipack_wks_dir")?;
	let wks_custom_dir = aipack_wks_dir.get_pack_custom_dir()?;
	let dest_dir = wks_custom_dir.join("test_ns_force").join("test_pack_force");
	std::fs::create_dir_all(dest_dir.path())?;
	save_file_content(&dest_dir.join("old_marker.txt"), "old content")?;

	// -- Exec
	// This will likely fail on the download/install step since test_ns_force@test_pack_force
	// is not a real pack, but we can verify the force trash behavior happened.
	let result = unpack_pack(dir_context, "test_ns_force@test_pack_force", true).await;

	// -- Check
	// The old marker file should have been trashed by the --force logic
	// (even if the overall operation fails due to pack not being found remotely)
	let old_marker = dest_dir.join("old_marker.txt");
	assert!(
		!old_marker.exists(),
		"Old marker file should have been removed by --force trash"
	);

	// The result may be an error (pack not found remotely or not installed),
	// which is expected for a fake pack identity.
	// The important thing is the force-trash happened.
	if result.is_err() {
		// Expected in test environment without real pack
	}

	// -- Cleanup
	if dest_dir.exists() {
		std::fs::remove_dir_all(dest_dir.path())?;
	}

	Ok(())
}
