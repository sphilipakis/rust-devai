use crate::tui::{RunsState, SumState};

#[derive(Default)]
pub struct AppState {
	sum_state: SumState,
	runs_state: RunsState,
	#[allow(unused)]
	pub run_id: Option<i64>,
}

impl AppState {
	pub fn mut_sum_state(&mut self) -> &mut SumState {
		&mut self.sum_state
	}
	pub fn mut_run_state(&mut self) -> &mut RunsState {
		&mut self.runs_state
	}
}
