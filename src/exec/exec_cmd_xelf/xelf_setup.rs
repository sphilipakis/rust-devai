use crate::Error; // Import Error type
use crate::Result;
use crate::cli::XelfSetupArgs;
use crate::dir_context::AipackBaseDir;
use crate::hub::get_hub;
use crate::init::extract_setup_aip_env_sh_zfile; // Import the specific function
use crate::init::init_base;
use simple_fs::{SPath, ensure_dir}; // Import ensure_dir and SPath
use std::fs; // Import fs for copy and write

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

	// Instructions for user
	let bin_dir = bin_dir.as_str();
	let path_to_env_sh = format!("$HOME/.aipack-base/bin/{}", target_env_script_path.name());
	hub.publish(format!(
		r#"
  IMPORTANT: Add '{bin_dir}' to your PATH environment. 
    You can add the "{path_to_env_sh}" in your sh file
      1)   On Mac: echo '\nsource "{path_to_env_sh}"' >> ~/.zshenv
      2) On Linux: echo 'source "{path_to_env_sh}"' >> ~/.bashrc

    Then, you can either; 
      - Start a new terminal
      - Or execute: source "{path_to_env_sh}"

    Then, check with
    - Run: which aip
    - You should see something like "path/to/home/.aipack-base/bin/aip"
"#
	))
	.await;

	hub.publish("\n==== 'self setup' completed ====\n").await;
	Ok(())
}

// region:    --- aip-env check & set

// endregion: --- aip-env check & set
