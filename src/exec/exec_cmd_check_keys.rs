use crate::Result;
use crate::exec::cli::CheckKeysArgs;
use crate::exec::support::{KEY_ENV_VARS, get_available_api_keys};
use crate::hub::get_hub;
use crate::tui_v1::PrintEvent;

/// Executes the check-keys command by getting available keys and publishing a PrintEvent.
pub async fn exec_check_keys(_args: CheckKeysArgs) -> Result<()> {
	// Get the set of available keys from the environment
	let available_keys = get_available_api_keys();

	// Create the print event
	let event = PrintEvent::ApiKeysStatus {
		all_keys: KEY_ENV_VARS,
		available_keys,
	};

	// Publish the event to the hub
	get_hub().publish(event).await;

	Ok(())
}
