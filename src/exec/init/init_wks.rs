use crate::Result;
use crate::dir_context::{AipackPaths, AipackWksDir, DirContext, find_wks_dir};
use crate::exec::init::init_assets;
use crate::hub::get_hub;
use crate::support::files::current_dir;
use simple_fs::{SPath, ensure_dir};
use std::fs::write;

// -- Doc Content
/// Note: The `show_info_always` will ensure that even if the `.aipack/` is found, it will print the message
///       This is useful for the `aip init` to always show the status
pub async fn init_wks(ref_dir: Option<&str>, show_info_always: bool) -> Result<DirContext> {
	let hub = get_hub();

	let wks_dir = if let Some(dir) = ref_dir {
		SPath::new(dir)
	} else if let Some(path) = find_wks_dir(current_dir()?)? {
		path
	} else {
		current_dir()?
	};

	let wks_dir = wks_dir.canonicalize()?;

	let aipack_paths = AipackPaths::from_wks_dir(&wks_dir)?;

	let aipack_wks_dir = aipack_paths
		.aipack_wks_dir()
		.ok_or_else(|| format!("Cannot Initialize Workspace because .aipack/ folder was not computed for {wks_dir}"))?;

	// -- Display the heading
	if aipack_wks_dir.exists() {
		if show_info_always {
			hub.publish("\n=== Initializing .aipack/").await;
			hub.publish(format!(
				"-- Parent path: '{wks_dir}'\n   (`.aipack/` already exists. Will create missing files)"
			))
			.await;
		}
	} else {
		hub.publish("\n=== Initializing .aipack/").await;
		hub.publish(format!(
			"-- Parent path: '{wks_dir}'\n   (`.aipack/` will be created at this path)"
		))
		.await;
	}

	// -- Init or refresh
	create_or_refresh_wks_files(aipack_wks_dir).await?;

	if show_info_always {
		hub.publish("=== DONE\n").await;
	}

	// -- Return
	let dir_context = DirContext::new(aipack_paths)?;

	Ok(dir_context)
}

/// Create or refresh missing files in a aipack directory
/// - create `.aipack/config.toml` if not present.
/// - ensure `.aipack/pack/custom/` to show use how to create per workspace agent pack
async fn create_or_refresh_wks_files(aipack_wks_dir: &AipackWksDir) -> Result<()> {
	let hub = get_hub();

	ensure_dir(aipack_wks_dir.path())?;

	// -- Create the config file
	let config_path = aipack_wks_dir.get_config_toml_path()?;

	if !config_path.exists() {
		let config_zfile = init_assets::extract_workspace_config_toml_zfile()?;
		write(&config_path, config_zfile.content)?;
		hub.publish(format!(
			"-> {:<18} '{}'",
			"Create config file",
			config_path.try_diff(aipack_wks_dir.parent().ok_or("Should have parent dir")?)?
		))
		.await;
	}

	// NOTE: Currently, we do not create the workspace .aipack/pack/custom directory because users can use their own paths to run agents.
	//       Eventually, we might support installing packs in the workspace using `aip install pro@coder --workspace`.
	//       These will be placed in `.aipack/pack/installed/` and will take precedence over the base custom & installed packs.

	// NOTE: For now, the workspace .aipack/pack/custom/ directory is still added to the pack resolution, which is beneficial for advanced users.

	Ok(())
}
