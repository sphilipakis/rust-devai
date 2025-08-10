use crate::store::rt_model::ErrRec;
use crate::support::time::every_kth_tick;
use crate::tui::core::AppState;

const TICK_LENGTH_DEFAULT: f64 = 0.2;

/// SysState & Metrics
impl AppState {
	/// NOTE: Might do a take later
	pub fn sys_err_rec(&self) -> Option<&ErrRec> {
		self.core.sys_err.as_ref()
	}

	pub fn is_kth_tick(&self, k: i64) -> bool {
		every_kth_tick(self.core.tick_now, TICK_LENGTH_DEFAULT, k)
	}

	pub fn is_even_tick(&self) -> bool {
		self.is_kth_tick(2)
	}

	pub fn show_sys_states(&self) -> bool {
		self.core.show_sys_states
	}

	pub fn memory(&self) -> u64 {
		self.core.memory
	}

	#[allow(unused)]
	pub fn cpu(&self) -> f64 {
		self.core.cpu
	}
}

// only for Core
impl AppState {
	/// Called from the app state processor on Shift M
	pub(in crate::tui::core) fn toggle_show_sys_states(&mut self) {
		self.core.show_sys_states = !self.core.show_sys_states;
	}

	/// Called every tick of the main loop (if show_sys_states)
	pub(in crate::tui::core) fn refresh_sys_state(&mut self) {
		let (memory, cpu) = self.core.sys_state.memory_and_cpu();
		self.core.memory = memory;
		self.core.cpu = cpu;
	}
}
