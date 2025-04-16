#![allow(unused)] // Now, as lot of upcoming code

use crate::cli::XelfUpdateArgs;
use crate::dir_context::AipackBaseDir;
use crate::hub::get_hub;
use crate::support::os; // Keep zip for now, might be used elsewhere or remove later if truly unused
use crate::{Error, Result};
use semver::Version;
use serde::Deserialize;
use simple_fs::ensure_dir;
use std::fs::{self, File};
use std::io::Write;
use std::process::Command; // Import Command

const LATEST_TOML_URL: &str = "https://repo.aipack.ai/aip-dist/stable/latest/latest.toml";
const DIST_BASE_URL: &str = "https://repo.aipack.ai/aip-dist/stable/latest";
const TMP_UPDATE_DIR_LEAF: &str = "bin/.tmp-update"; // Directory for download and extraction
const DOWNLOAD_FILE_NAME: &str = "aip.tar.gz";

#[derive(Deserialize, Debug)]
struct LatestStable {
	version: String,
}

#[derive(Deserialize, Debug)]
struct LatestTomlData {
	latest_stable: LatestStable,
}

pub async fn exec_xelf_update(_args: XelfUpdateArgs) -> Result<()> {
	let hub = get_hub();

	hub.publish("Checking for the latest AIPACK version...\n").await;

	// -- Fetch latest version info
	let client = reqwest::Client::new();
	let resp = client.get(LATEST_TOML_URL).send().await?;

	if !resp.status().is_success() {
		hub.publish(Error::custom(format!(
			"Failed to fetch latest version info from {}. Status: {}",
			LATEST_TOML_URL,
			resp.status()
		)))
		.await;
		return Ok(());
	}

	let toml_content = resp.text().await?;

	// -- Parse TOML
	let latest_data: LatestTomlData = match toml::from_str(&toml_content) {
		Ok(data) => data,
		Err(e) => {
			hub.publish(Error::custom(format!(
				"Failed to parse latest version TOML from {}. Cause: {}",
				LATEST_TOML_URL, e
			)))
			.await;
			return Ok(());
		}
	};

	let latest_version_str = &latest_data.latest_stable.version;
	let latest_version = match Version::parse(latest_version_str) {
		Ok(ver) => ver,
		Err(e) => {
			hub.publish(Error::custom(format!(
				"Failed to parse latest version '{}'. Cause: {}",
				latest_version_str, e
			)))
			.await;
			return Ok(());
		}
	};

	let current_version = match Version::parse(crate::VERSION) {
		Ok(ver) => ver,
		Err(e) => {
			hub.publish(Error::custom(format!(
				"Failed to parse current version '{}'. Cause: {}",
				crate::VERSION,
				e
			)))
			.await;
			return Ok(());
		}
	};

	if current_version < latest_version {
		hub.publish(format!(
			"You need to update your version.\nYou have version '{}', and the latest version is '{}'.\nGo to {} to update.",
			current_version, latest_version, "https://aipack.ai/doc/install"
		))
		.await;
	} else {
		hub.publish(format!(
			"You have the latest version '{}'. (current latest version '{}')\n\nAll Good, No need to update.",
			current_version, latest_version,
		))
		.await;
	}

	Ok(())
}

// region:    --- Upcoming code

// NOT FULLY IMPLEMENTED

pub async fn exec_xelf_update_to_activate(_args: XelfUpdateArgs) -> Result<()> {
	let hub = get_hub();
	hub.publish("\nChecking for latest AIP version...").await;

	// -- Fetch latest version info
	let client = reqwest::Client::new();
	let resp = client.get(LATEST_TOML_URL).send().await?;

	if !resp.status().is_success() {
		return Err(Error::custom(format!(
			"Failed to fetch latest version info from {}. Status: {}",
			LATEST_TOML_URL,
			resp.status()
		)));
	}

	let toml_content = resp.text().await?;

	// -- Parse TOML
	let latest_data: LatestTomlData = toml::from_str(&toml_content).map_err(|e| {
		Error::custom(format!(
			"Failed to parse latest version TOML from {}. Cause: {}",
			LATEST_TOML_URL, e
		))
	})?;

	let latest_version_str = &latest_data.latest_stable.version;
	let latest_version = Version::parse(latest_version_str).map_err(|e| {
		Error::custom(format!(
			"Failed to parse latest version '{}'. Cause: {}",
			latest_version_str, e
		))
	})?;
	let current_version = Version::parse(crate::VERSION).map_err(Error::custom)?;

	hub.publish(format!(
		"-> Current version: {}, Latest version: {}",
		current_version, latest_version
	))
	.await;

	// -- Compare versions
	if latest_version > current_version {
		hub.publish(format!("\nNew version {} available. Downloading...", latest_version))
			.await;

		let platform = os::get_platform();
		if platform == "unknown-unknown" {
			return Err(Error::custom("Cannot update: Unsupported OS/Architecture combination."));
		}

		// -- Prepare download and extraction paths
		let base_dir = AipackBaseDir::new()?;
		let tmp_update_dir = base_dir.join(TMP_UPDATE_DIR_LEAF);
		let download_file_path = tmp_update_dir.join(DOWNLOAD_FILE_NAME);

		// -- Clean previous update attempt (if any)
		if tmp_update_dir.exists() {
			fs::remove_dir_all(&tmp_update_dir)?;
			hub.publish(format!(
				"-> Cleaned previous temporary update directory: {}",
				tmp_update_dir
			))
			.await;
		}
		ensure_dir(&tmp_update_dir)?; // Create the temporary directory

		// -- Construct download URL
		let download_url = format!("{}/{}/{}", DIST_BASE_URL, platform, DOWNLOAD_FILE_NAME);

		hub.publish(format!("-> Downloading from {}...", download_url)).await;

		// -- Download the file
		let mut resp = client.get(&download_url).send().await?;
		if !resp.status().is_success() {
			return Err(Error::custom(format!(
				"Failed to download update package from {}. Status: {}",
				download_url,
				resp.status()
			)));
		}

		let mut dest = File::create(&download_file_path)?;
		while let Some(chunk) = resp.chunk().await? {
			dest.write_all(&chunk)?;
		}
		hub.publish(format!("-> Downloaded update to {}", download_file_path)).await;

		// -- Extract the tar.gz file using tar command into the same tmp directory
		hub.publish(format!(
			"-> Extracting {} into {} using tar...",
			download_file_path, tmp_update_dir
		))
		.await;

		// Ensure download_file_path and tmp_update_dir are valid UTF-8 strings
		let download_file_str = download_file_path.as_str();
		let tmp_update_dir_str = tmp_update_dir.as_str();

		// Run the tar command
		let output = Command::new("tar")
			.args(["-xzf", download_file_str, "-C", tmp_update_dir_str]) // Extract into tmp_update_dir
			.output()
			.map_err(|e| Error::custom(format!("Failed to execute tar command: {}", e)))?;

		if !output.status.success() {
			let stderr = String::from_utf8_lossy(&output.stderr);
			// Clean up the failed attempt dir
			let _ = fs::remove_dir_all(&tmp_update_dir);
			return Err(Error::custom(format!(
				"Failed to extract tar file '{}'. tar command stderr: {}",
				download_file_path, stderr
			)));
		}

		// Optionally remove the tar.gz file after successful extraction
		// fs::remove_file(&download_file_path)?;

		hub.publish("-> Extraction complete.".to_string()).await;

		// --- Next steps would be here:
		// 1. Find the 'aip' executable within `tmp_update_dir`.
		// 2. Determine the target path: `base_dir.join("bin/aip")`.
		// 3. Handle platform differences (.exe suffix for Windows).
		// 4. Move/replace the old binary with the new one (atomically if possible, might need renaming).
		// 5. Set executable permissions on Unix-like systems.
		// 6. Clean up `tmp_update_dir`.
		// 7. Inform the user about completion or restart requirement.

		hub.publish("\nUpdate downloaded and extracted. Manual installation steps are pending implementation.")
			.await;
		hub.publish(format!("   New version located in: {}", tmp_update_dir)).await;
	} else {
		hub.publish("\nYou are running the latest version of AIP.").await;
	}

	Ok(())
}

// endregion: --- Upcoming code

