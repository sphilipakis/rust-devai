use crate::store::rt_model::Run;
use crate::tui::SumState;

#[derive(Default)]
pub struct AppState {
	sum_state: SumState,

	pub run_idx: Option<i32>,

	// newest to oldest
	pub runs: Vec<Run>,
}

impl AppState {
	pub fn mut_sum_state(&mut self) -> &mut SumState {
		&mut self.sum_state
	}

	pub fn current_run(&self) -> Option<&Run> {
		if let Some(idx) = self.run_idx {
			self.runs.get(idx as usize)
		} else {
			None
		}
	}
}
