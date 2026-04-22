use crate::support::time::tick_count;
use crate::model::Id;
use crate::tui::core::{AppState, RunItem, RunTab};
use crate::tui::support::offset_and_clamp_option_idx_in_len;

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

	/// Move the run selection by `offset` within the currently visible nav list.
	/// This keeps keyboard navigation aligned with the visible rows so collapsed
	/// sub-run branches are skipped.
	pub fn offset_run_idx_in_visible_nav(&mut self, offset: i32) {
		let visible_ids: Vec<Id> = self.visible_run_items_for_nav().iter().map(|r| r.id()).collect();
		let len = visible_ids.len();
		if len == 0 {
			return;
		}

		let current_run_id = self.current_run_item().map(|r| r.id());
		let current_visible_idx: Option<i32> = current_run_id
			.and_then(|id| visible_ids.iter().position(|vid| *vid == id))
			.map(|i| i as i32);

		let new_idx = offset_and_clamp_option_idx_in_len(&current_visible_idx, offset, len);
		if let Some(new_idx) = new_idx
			&& let Some(target_id) = visible_ids.get(new_idx as usize).copied()
		{
			self.set_run_id(target_id);
		}
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
