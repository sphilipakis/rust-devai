use crate::tui::{MainState, SumState};

#[derive(Default)]
pub struct AppState {
	sum_state: SumState,
	run_state: MainState,
}

impl AppState {
	pub fn mut_sum_state(&mut self) -> &mut SumState {
		&mut self.sum_state
	}
	pub fn mut_run_state(&mut self) -> &mut MainState {
		&mut self.run_state
	}
}
