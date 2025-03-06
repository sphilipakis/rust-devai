use crate::tui::{PrintEvent, print_pack_list};
use std::sync::Arc;

pub fn handle_print(print_event: Arc<PrintEvent>) {
	match &*print_event {
		PrintEvent::PackList(pack_dirs) => {
			let pack_dirs: Vec<&_> = pack_dirs.iter().collect();
			print_pack_list(&pack_dirs)
		}
	}
}
