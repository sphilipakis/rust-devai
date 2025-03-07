use crate::tui::PrintEvent;
use crate::tui::printer::printers::{self};
use std::sync::Arc;

pub fn handle_print(print_event: Arc<PrintEvent>, interactive: bool) {
	match &*print_event {
		PrintEvent::PackList(pack_dirs) => {
			let pack_dirs: Vec<&_> = pack_dirs.iter().collect();
			printers::print_pack_list(&pack_dirs, interactive)
		}
	}
}
