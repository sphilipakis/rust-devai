use crate::tui::core::{AppState, RunItem, RunTab};

/// RunsView
impl AppState {
	pub fn run_idx(&self) -> Option<usize> {
		self.core.run_idx.map(|idx| idx as usize)
	}

	pub fn set_run_idx(&mut self, idx: Option<usize>) {
		if let Some(idx) = idx {
			self.core.set_run_by_idx(idx as i32);
		}
	}

	pub fn run_items(&self) -> &[RunItem] {
		&self.core.run_items
	}

	pub fn current_run_item(&self) -> Option<&RunItem> {
		if let Some(idx) = self.core.run_idx {
			self.core.run_items.get(idx as usize)
		} else {
			None
		}
	}

	pub fn run_tab(&self) -> RunTab {
		self.core.run_tab
	}

	pub fn set_run_tab(&mut self, run_tab: RunTab) {
		self.core.run_tab = run_tab;
	}
}
