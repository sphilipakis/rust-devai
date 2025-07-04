use crate::tui_v1::PrintEvent;
use crate::tui_v1::printer::printers::{self, print_error_generic, print_key_env_missing};
use std::sync::Arc;

pub fn handle_print(print_event: Arc<PrintEvent>, interactive: bool) {
	// TODO: Need to add proper error handling for the print functions
	match &*print_event {
		// -- Print pack list (aip list)
		PrintEvent::PackList(pack_dirs) => {
			let pack_dirs: Vec<&_> = pack_dirs.iter().collect();
			printers::print_pack_list(&pack_dirs, interactive);
		}

		PrintEvent::InfoShort(info) => {
			let _ = printers::print_info_short(info);
		}
		// -- Print api key status (aip check-keys)
		PrintEvent::ApiKeysStatus {
			all_keys,
			available_keys,
		} => {
			let _ = printers::print_api_keys(all_keys, available_keys);
		}

		// -- Print API Key Missing
		PrintEvent::ApiKeyEnvMissing { model_iden, env_name } => {
			print_key_env_missing(env_name, model_iden, interactive)
		}

		// Print Generic error
		PrintEvent::GenericErrorMsg(msg) => print_error_generic(msg, interactive),
	}
}
