// region:    --- Modules

use crate::Result;
use crate::dir_context::AipackBaseDir;
use crate::exec::exec_cmd_xelf::support::{get_aip_stable_url, has_aip_in_path};
use crate::hub::{HubEvent, get_hub};
use crate::support::proc::{self, ProcOptions};
use crate::support::webc;
use semver::Version;
use simple_fs::ensure_dir; // Assuming this module provides ensure_dir and write_to_file

// endregion: --- Modules

// region:    --- Constants

const ARCHIVE_NAME: &str = "aip.tar.gz";

// endregion: --- Constants

// region:    --- Public Functions

/// Executes the update process for non-Windows (Nix-like) systems.
pub(super) async fn exec_update_for_nix(remote_version: &Version, is_latest: bool) -> Result<()> {
	let hub = get_hub();
	hub.publish(format!("Starting update to version {remote_version}...")).await;

	// -- Get base dir and define paths
	// The AipackBaseDir::init_from_env_or_home will resolve '~' and environment variables.
	let aipack_base_dir = AipackBaseDir::new()?;
	// let base_dir = aipack_base_dir.path();
	let tmp_dir = aipack_base_dir.bin_tmp_dir();
	let archive_path = tmp_dir.join(ARCHIVE_NAME);

	// -- Ensure tmp directory exists
	// Assuming files::ensure_dir creates the directory if it doesn't exist.
	ensure_dir(&tmp_dir)?;
	hub.publish(format!("Using temporary directory: {tmp_dir}")).await;

	let proc_opts = ProcOptions::default().with_cwd(&tmp_dir);

	// -- Download
	hub.publish(format!("Downloading new version ({remote_version})...")).await;
	let download_url = if is_latest {
		get_aip_stable_url(None)?
	} else {
		get_aip_stable_url(Some(remote_version))?
	};

	webc::web_download_to_file(&download_url, &archive_path).await?;

	// -- Extract
	hub.publish(format!("Extracting {ARCHIVE_NAME} in {tmp_dir}...")).await;
	proc::proc_exec("tar", &["-xvf", ARCHIVE_NAME], Some(&proc_opts)).await?;
	hub.publish("Extraction complete.").await;

	// -- Run setup for the new version
	// Assumes the executable in the archive is named 'aip' and is placed in tmp_dir directly by tar.
	let new_aip_exe_path = tmp_dir.join("aip");
	hub.publish(format!("Running setup for the new version using {new_aip_exe_path}..."))
		.await;

	let setup_stdout =
		proc::proc_exec_to_output(new_aip_exe_path.as_str(), &["self", "setup"], Some(&proc_opts)).await?;
	let setup_str = format!("'aip self setup' executed:\n{setup_stdout}\n");
	hub.publish(setup_str).await;

	hub.publish(HubEvent::info_short(format!(
		"Update successful! New version 'v{remote_version}' installed.\n"
	)))
	.await;

	if !has_aip_in_path() {
		hub.publish(HubEvent::info_short(
		"It seems you do not have '~/.aipack-base/bin' in path.
Please restart your terminal session or source your shell profile (e.g., `source ~/.bashrc`, `source ~/.zshrc`) for changes to take effect.		
		"
	))
	.await;
	}

	// -- Cleanup (Optional: User can uncomment to activate)
	// hub.publish(format!("Cleaning up temporary directory: {}", tmp_dir)).await;
	// if let Err(e) = std::fs::remove_dir_all(&tmp_dir) {
	//     // This is not critical, so just log a warning.
	//     hub.publish(format!("Warning: Failed to remove temporary directory {}: {}", tmp_dir, e)).await;
	// }

	Ok(())
}

// endregion: --- Public Functions
