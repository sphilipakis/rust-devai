use crate::Result;
use crate::cli::XelfSetupArgs;
use crate::dir_context::AipackBaseDir;
use crate::hub::get_hub;
use crate::hub::hub_prompt;
use crate::init::extract_setup_aip_env_sh_zfile; // Import the specific function
use crate::init::init_base;
use crate::support::os;
use crate::support::os::current_os;
use simple_fs::read_to_string;
use simple_fs::{SPath, ensure_dir}; // Import ensure_dir and SPath
use std::fs;
use std::fs::remove_file;
use std::fs::write;

// Because the bin with .aip
const BIN_DIR: &str = "bin";

/// Executes the `self setup` command.
pub async fn exec_xelf_setup(_args: XelfSetupArgs) -> Result<()> {
	// First init the base `~/.aipack-base/`
	init_base(false).await?;
	let aipack_base_dir = AipackBaseDir::new()?;
	let hub = get_hub();

	hub.publish(format!(
		"\n==== Executing 'self setup' ({}) ====\n",
		aipack_base_dir.path()
	))
	.await;

	// -- Create the bin directory
	let base_bin_dir = aipack_base_dir.join(BIN_DIR);
	if ensure_dir(&base_bin_dir)? {
		hub.publish(format!("-> {:<18} '{}'", "Create dir", base_bin_dir)).await;
	}

	// -- Copy current executable
	let current_exe = std::env::current_exe()?;
	let current_exe_spath = SPath::from_std_path_buf(current_exe)?;

	// Check if already running from within the base bin directory (or subdirs)
	let is_current_exe_at_base_bin_dir = current_exe_spath.as_str().starts_with(base_bin_dir.as_str());

	// If running on the already installed, just warn that it will just udpate the settings
	let tmp_exe_to_trash = if is_current_exe_at_base_bin_dir {
		hub.publish(format!(
			"WARN: Running 'self setup' on installed 'aip' at  '{}'. This will just update the settings (but not update the 'aip' binary)",
			aipack_base_dir.as_str()
		))
		.await;
		None
	}
	// if running on another aip, then, perform the copy
	else {
		let target_exe_path = base_bin_dir.join(current_exe_spath.name());
		// Copy the file
		fs::copy(&current_exe_spath, &target_exe_path)?;
		hub.publish(format!(
			"-> {:<18} '{}' to '{}'",
			"Copy executable", current_exe_spath, target_exe_path
		))
		.await;
		Some(current_exe_spath)
	};

	if os::is_unix() {
		unix_setup_env(&base_bin_dir).await?;
	} else {
		#[cfg(windows)]
		for_windows::windows_setup_env(&base_bin_dir).await?;
	}

	// -- Eventually remove the current exec
	// NOTE: Only if there is a `.tar.gz` sibling
	if let Some(tmp_exe_to_trash) = tmp_exe_to_trash {
		let gz_sibling = tmp_exe_to_trash.new_sibling(format!("{}.tar.gz", tmp_exe_to_trash.stem()));
		if gz_sibling.exists() && tmp_exe_to_trash.stem() == "aip" {
			// Here we delete directly, as calling safer_trash_file will prompt a Mac finder access dialog which would be confusing
			remove_file(&tmp_exe_to_trash)?;
			hub.publish(format!("-> {:<18} '{}'", "Temp aip file deleted", tmp_exe_to_trash))
				.await;
		}
	}

	hub.publish("\n==== 'self setup' completed ====\n").await;
	Ok(())
}

// region:    --- Window Setup

#[cfg(windows)]
mod for_windows {
	use super::*;

	use std::process::Command;

	#[cfg(windows)]
	use std::os::windows::process::CommandExt;

	pub async fn windows_setup_env(base_bin_dir: &SPath) -> Result<()> {
		let hub = get_hub();

		// Get current user PATH
		let current_path = std::env::var("PATH").unwrap_or_default();

		// Check if path is already present (case-insensitive on Windows)
		let found_path = current_path.split(';').find(|p| p.contains(".aipack-base"));

		if let Some(found_path) = found_path {
			hub.publish(format!(".aipack-base path already setup. ({found_path})")).await;
			return Ok(());
		}

		let new_path = base_bin_dir.as_str().replace("/", "\\");

		let user_response = hub_prompt(
			hub,
			format!("\nDo you want to add '{new_path}' to your shell PATH?: Y/n "),
		)
		.await?;

		if user_response.trim() != "Y" {
			hub.publish(format!(
				r#"-! Answer was not 'Y' so skipping updating environment path.
   Make sure to add '{new_path}' in your system path.
	 Then, you can run: aip -V
	 To check the version of aipack"#
			))
			.await;
			return Ok(());
		}

		// Append new path
		add_new_path(&new_path).await?;

		hub.publish(
			r#"
Setup should be complete now
  - You need to start a new terminal
  - If you are in VSCode, it has to be restarted for the new Path to take effect
    (restart the VSCode terminal is not enough :(

Then, check with
  - Run: aip -V 
  - Should print something like "aipack 0.6.18"
"#,
		)
		.await;

		Ok(())
	}

	async fn add_new_path(new_path: &str) -> Result<()> {
		let hub = get_hub();

		let current_user_path = exec_powershell(r#"[Environment]::GetEnvironmentVariable("Path", "User")"#)?;
		let current_user_path = current_user_path.trim();

		let updated_path = format!("{};{}", current_user_path.trim_end_matches(';'), new_path);
		let power_set_path_cmd = &format!(
			"[Environment]::SetEnvironmentVariable('Path', '{}', 'User')",
			updated_path.replace("'", "''") // escape single quotes
		);
		exec_powershell(power_set_path_cmd)?;

		hub.publish(format!(
			"-> Added to shell path (with [Environment]::SetEnvironmentVariable('Path'): '{new_path}'"
		))
		.await;

		Ok(())
	}

	fn exec_powershell(power_cmd: &str) -> Result<String> {
		let mut cmd = Command::new("powershell");
		cmd.args(["-NoProfile", "-Command", power_cmd]);

		#[cfg(windows)]
		cmd.creation_flags(0x08000000); // create no window

		let output = cmd.output()?;

		if output.status.success() {
			let stdout = String::from_utf8_lossy(&output.stdout);
			Ok(stdout.trim().to_string())
		} else {
			let stderr = String::from_utf8_lossy(&output.stderr);
			Err(format!("Cannot execute command '{power_cmd}'. Cause: {stderr}").into())
		}
	}
}

// endregion: --- Window Setup

// region:    --- Unix Setup

async fn unix_setup_env(base_bin_dir: &SPath) -> Result<()> {
	let hub = get_hub();

	// -- Extract and copy aip-env
	// Note: Assuming the zip file contains the path "_setup/aip-env" directly at the root.
	let env_script_zfile = extract_setup_aip_env_sh_zfile()?;
	let target_env_script_path = base_bin_dir.join("aip-env");
	fs::write(&target_env_script_path, env_script_zfile.content)?;

	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt as _; // Import fs for copy and write
		if os::is_linux() || os::is_mac() {
			let mut perms = fs::metadata(&target_env_script_path)?.permissions();
			perms.set_mode(0o755); // rwxr-xr-x
			fs::set_permissions(&target_env_script_path, perms)?;
		}
	}
	hub.publish(format!("-> {:<18} '{}'", "Create script", target_env_script_path))
		.await;

	// -- Check & Setup env
	if let Some(home_sh_env_path) = os::get_os_env_file_path() {
		if home_sh_env_path.exists() {
			// -- get the home sh env content or empty string if not
			// NOTE: This will eventually create the file is not present
			let content = if home_sh_env_path.exists() {
				read_to_string(&home_sh_env_path)?
			} else {
				"".to_string()
			};

			if content.contains(".aipack-base") {
				hub.publish(format!(
					"-! {:<18} '{home_sh_env_path}' seems to have the .aipack-base path setup. So, skipping further setup.",
					"Setup PATH"
				))
				.await;
			}
			// -- Create the file if we have a os_source_line for this OS
			else if let Some(os_source_line) = os_source_line(&target_env_script_path) {
				let action_str = if content.is_empty() { "create" } else { "update" };

				let user_response = hub_prompt(
					hub,
					format!(
						"\nDo you want to {action_str} the '{home_sh_env_path}' with the required aipack-base/bin path: Y/n "
					),
				)
				.await?;
				if user_response.trim() == "Y" {
					let content = format!("{}\n\n{}\n", content.trim_end(), os_source_line);
					write(&home_sh_env_path, content)?;
					hub.publish(format!(
						"-> {:<18} Added '{os_source_line}' in file '{home_sh_env_path}'",
						"Setup PATH"
					))
					.await;
				} else {
					hub.publish(format!(
						"-! Answer was not 'Y' so skipping updating '{home_sh_env_path}'"
					))
					.await;
				}
			} else {
				hub.publish(format!(
					"-! No source line for the current OS. Skipping updating '{home_sh_env_path}'"
				))
				.await;
			}
		}
	}

	// -- Print final message
	let path_to_aip_env_sh = format!("$HOME/.aipack-base/bin/{}", target_env_script_path.name());
	hub.publish(format!(
		r#"
Setup should be complete now
  - You need to start a new terminal
  - Or execute: source "{path_to_aip_env_sh}"

Then, check with
  - Run: which aip
  - You should see something like "path/to/home/.aipack-base/bin/aip"
  - Run: aip -V 
  - Should print something like "aipack 0.6.17"
"#
	))
	.await;

	Ok(())
}
// endregion: --- Unix Setup

// region:    --- aip-env check & set

fn os_source_line(target_env_script_path: &SPath) -> Option<String> {
	let path_to_aip_env_sh = format!("$HOME/.aipack-base/bin/{}", target_env_script_path.name());
	match current_os() {
		os::OsType::Mac => Some(format!("source \"{path_to_aip_env_sh}\"")),
		os::OsType::Linux => Some(format!("source \"{path_to_aip_env_sh}\"")),
		os::OsType::Windows => None,
		os::OsType::Unknown => None,
	}
}

// endregion: --- aip-env check & set
