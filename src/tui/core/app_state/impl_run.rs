use crate::support::time::tick_count;
use crate::model::Id;
use crate::tui::core::{AppState, RunItem, RunTab};

/// RunsView
impl AppState {
	pub fn run_idx(&self) -> Option<usize> {
		self.core.run_idx.map(|idx| idx as usize)
	}

	pub fn running_tick_count(&self) -> Option<i64> {
		let running_start = self.core().running_tick_start?;

		let duration_micro = (self.core().time - running_start).max(0);
		let ticks = tick_count(duration_micro, 0.2);

		Some(ticks)
	}

	/// Running tick flag (true/false) when running
	pub fn running_tick_flag(&self) -> Option<bool> {
		let ticks = self.running_tick_count()?;

		Some((ticks / 3) % 2 == 0)
	}

	pub fn set_run_idx(&mut self, idx: Option<usize>) {
		if let Some(idx) = idx {
			self.core.set_run_by_idx(idx as i32);
		}
	}

	pub fn set_run_id(&mut self, run_id: Id) {
		self.core.set_run_by_id(run_id);
	}

	pub fn run_items(&self) -> &[RunItem] {
		self.core.run_item_store.items()
	}

	pub fn current_run_item(&self) -> Option<&RunItem> {
		if let Some(idx) = self.core.run_idx {
			self.core.run_item_store.items().get(idx as usize)
		} else {
			None
		}
	}

	pub fn current_root_run_id(&self) -> Option<Id> {
		let run_item = self.current_run_item()?;

		if run_item.is_root() {
			Some(run_item.id())
		} else {
			run_item.ancestors().first().copied()
		}
	}

	pub fn visible_run_items_for_nav<'a>(&'a self) -> Vec<&'a RunItem> {
		self.core.run_item_store.visible_items_for_root_branch(self.current_root_run_id())
	}

	#[allow(unused)]
	pub fn all_run_children<'a>(&'a self, run_item: &RunItem) -> Vec<&'a RunItem> {
		self.core.run_item_store.all_children(run_item)
	}

	#[allow(unused)]
	pub fn is_root_run(&self, run_item: &RunItem) -> bool {
		run_item.is_root()
	}

	pub fn run_tab(&self) -> RunTab {
		self.core.run_tab
	}

	pub fn set_run_tab(&mut self, run_tab: RunTab) {
		self.core.run_tab = run_tab;
	}
}
