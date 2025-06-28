use semver::Version;

use crate::VERSION;
use crate::error::{Error, Result};
use crate::exec::cli::XelfUpdateArgs;
use crate::hub::{HubEvent, get_hub};
use serde::Deserialize; // Current CLI version

const LATEST_TOML_URL: &str = "https://repo.aipack.ai/aip-dist/stable/latest/latest.toml";

pub async fn exec_xelf_update(args: XelfUpdateArgs) -> Result<()> {
	let hub = get_hub();
	hub.publish(HubEvent::info_short(format!("Current aip version: {VERSION}")))
		.await;

	hub.publish(HubEvent::info_short("Checking for latest version...")).await;

	let (target_version, explicit_version) = if let Some(version) = args.version {
		let version = version.strip_prefix("v").unwrap_or(&version);
		let version = Version::parse(version).map_err(|err| format!("Version '{version}' not valid. {err}"))?;
		(version, true)
	} else {
		match fetch_latest_remote_version().await {
			Ok(latest_version) => {
				hub.publish(HubEvent::info_short(format!(
					"Latest remote version available: {latest_version}"
				)))
				.await;
				(latest_version, false)
			}
			Err(e) => {
				hub.publish(HubEvent::Error { error: e.into() }).await;
				hub.publish(
					"Failed to check for updates. Please check your internet connection or try again later.\n\
						You can manually check for releases at: https://aipack.ai/doc/install",
				)
				.await;
				return Ok(());
			}
		}
	};

	// Parse current and remote versions for comparison.
	// Handles cases like "0.7.11-WIP" by parsing the "0.7.11" part.
	let current_v = Version::parse(VERSION)
		.map_err(|e| Error::custom(format!("Failed to parse current version string '{VERSION}': {e}")))?;

	if explicit_version || target_version > current_v {
		if !explicit_version && target_version > current_v {
			hub.publish(format!(
				"A new version {target_version} is available (you have v{current_v})."
			))
			.await;
		} else if explicit_version {
			hub.publish(format!(
				"Installing v{target_version} is available (you have v{current_v})."
			))
			.await;
		}

		// Check OS for platform-specific update logic
		if cfg!(target_os = "windows") {
			hub.publish(
				"Automatic update is not yet supported on Windows.\n\
				Please update manually by downloading the latest release from:\n\
				https://aipack.ai/doc/install
				",
			)
			.await;
		} else {
			// -- Execute update for non-Windows (Nix-like) systems
			match super::xelf_update_nix::exec_update_for_nix(&target_version, true).await {
				Ok(_) => {
					// Success message is handled within exec_update_for_nix
				}
				Err(e) => {
					// Publish the specific error that occurred during the update attempt
					hub.publish(HubEvent::Error { error: e.into() }).await;
					// Provide fallback instructions
					hub.publish(
						"Automatic update failed. Please try updating manually from:\n\
						https://aipack.ai/doc/install",
					)
					.await;
				}
			}
		}
	} else {
		hub.publish(HubEvent::info_short(format!(
			"You are already using the latest version ({current_v})"
		)))
		.await;
	}

	Ok(())
}

// region:    --- Private Functions

#[derive(Deserialize, Debug)]
struct LatestStable {
	version: String,
}

#[derive(Deserialize, Debug)]
struct LatestTomlData {
	latest_stable: LatestStable,
}

/// Fetches the latest remote version string.
/// Placeholder: In a real implementation, this would fetch from GitHub API or a dedicated endpoint.
async fn fetch_latest_remote_version() -> Result<Version> {
	// -- Fetch latest version info
	let client = reqwest::Client::new();
	let resp = client.get(LATEST_TOML_URL).send().await?;

	if !resp.status().is_success() {
		return Err(Error::custom(format!(
			"Failed to fetch latest version info from {LATEST_TOML_URL}. Status: {}",
			resp.status()
		)));
	}

	let toml_content = resp.text().await?;

	// -- Parse TOML
	let latest_data: LatestTomlData = toml::from_str(&toml_content).map_err(|e| {
		Error::custom(format!(
			"Failed to parse latest version TOML from {LATEST_TOML_URL}. Cause: {e}"
		))
	})?;

	let latest_version_str = &latest_data.latest_stable.version;
	let latest_version = Version::parse(latest_version_str).map_err(|e| {
		Error::custom(format!(
			"Failed to parse latest version '{latest_version_str}'. Cause: {e}"
		))
	})?;

	Ok(latest_version)
}

// endregion: --- Private Functions
