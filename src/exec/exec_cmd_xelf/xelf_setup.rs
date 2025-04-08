use crate::Error; // Import Error type
use crate::Result;
use crate::cli::XelfSetupArgs;
use crate::dir_context::AipackBaseDir;
use crate::hub::get_hub;
use crate::hub::hub_prompt;
use crate::init::extract_setup_aip_env_sh_zfile; // Import the specific function
use crate::init::init_base;
use crate::support::os;
use crate::support::os::current_os;
use crate::support::os::is_linux;
use crate::support::os::is_mac;
use simple_fs::read_to_string;
use simple_fs::{SPath, ensure_dir}; // Import ensure_dir and SPath
use std::fs;
use std::fs::write;
use std::os::unix::fs::PermissionsExt as _; // Import fs for copy and write

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
	let bin_dir = aipack_base_dir.join(BIN_DIR);
	if ensure_dir(&bin_dir)? {
		hub.publish(format!("-> {:<18} '{}'", "Create dir", bin_dir)).await;
	}

	// -- Extract and copy aip-env
	// Note: Assuming the zip file contains the path "_setup/aip-env" directly at the root.
	let env_script_zfile = extract_setup_aip_env_sh_zfile()?;
	let target_env_script_path = bin_dir.join("aip-env");
	fs::write(&target_env_script_path, env_script_zfile.content)?;
	if is_linux() || is_mac() {
		let mut perms = fs::metadata(&target_env_script_path)?.permissions();
		perms.set_mode(0o755); // rwxr-xr-x
		fs::set_permissions(&target_env_script_path, perms)?;
	}
	hub.publish(format!("-> {:<18} '{}'", "Create script", target_env_script_path))
		.await;

	// -- Copy current executable
	let current_exe = std::env::current_exe()?;
	let current_exe_spath = SPath::from_std_path_buf(current_exe)?;

	// Check if already running from within the base bin directory (or subdirs)
	if current_exe_spath.as_str().starts_with(bin_dir.as_str()) {
		return Err(Error::custom(format!(
			"Cannot run 'self setup' from the installation directory ('{}').\nRun it from the downloaded executable's location.",
			bin_dir
		)));
	}

	// Check if trying to copy from anywhere within the base dir (less strict than above)
	if current_exe_spath.as_str().starts_with(aipack_base_dir.as_str()) {
		hub.publish(format!(
			"WARN: Running 'self setup' from within '{}'. This might indicate an unusual setup.",
			aipack_base_dir.as_str()
		))
		.await;
		// Decide whether to error out or just warn. Let's warn for now.
		// return Err(Error::custom(format!(
		//     "Cannot run 'self setup' from within the aipack base directory ('{}'). Run it from the downloaded executable's location.",
		//     aipack_base_dir.as_str()
		// )));
	}

	let target_exe_path = bin_dir.join("aip");
	// Copy the file
	fs::copy(&current_exe_spath, &target_exe_path)?;
	hub.publish(format!(
		"-> {:<18} '{}' to '{}'",
		"Copy executable", current_exe_spath, target_exe_path
	))
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

	hub.publish("\n==== 'self setup' completed ====\n").await;
	Ok(())
}

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
