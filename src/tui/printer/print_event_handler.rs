use crate::tui::PrintEvent;
use crate::tui::printer::printers::{self};
use std::sync::Arc;

pub fn handle_print(print_event: Arc<PrintEvent>, interactive: bool) {
	// TODO: Need to add proper error handling for the print functions
	match &*print_event {
		PrintEvent::PackList(pack_dirs) => {
			let pack_dirs: Vec<&_> = pack_dirs.iter().collect();
			printers::print_pack_list(&pack_dirs, interactive);
		}
		PrintEvent::ApiKeysStatus {
			all_keys,
			available_keys,
		} => {
			let _ = printers::print_api_keys(all_keys, available_keys);
		}
	}
}
