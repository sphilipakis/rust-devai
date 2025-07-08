use crate::store::ModelManager;
use crate::store::rt_model::{Run, Task};
use crate::tui::event::LastAppEvent;

/// The global app state
/// IMPORTANT: We define it in this file so that some state can be private
pub struct AppState {
	pub(in crate::tui) run_idx: Option<i32>,
	pub(in crate::tui) log_scroll: u16,

	// newest to oldest
	pub(in crate::tui) runs: Vec<Run>,

	pub(in crate::tui) tasks: Vec<Task>,

	pub(in crate::tui) mm: ModelManager,
	pub(in crate::tui) last_app_event: LastAppEvent,
}

/// Contrustor
impl AppState {
	pub fn new(mm: ModelManager, last_app_event: LastAppEvent) -> Self {
		Self {
			run_idx: None,
			log_scroll: 0,
			runs: Vec::new(),
			tasks: Vec::new(),
			mm,
			last_app_event,
		}
	}
}

/// Getter
impl AppState {
	pub fn run_idx(&self) -> Option<usize> {
		self.run_idx.map(|idx| idx as usize)
	}

	pub fn current_run(&self) -> Option<&Run> {
		if let Some(idx) = self.run_idx {
			self.runs.get(idx as usize)
		} else {
			None
		}
	}

	pub fn runs(&self) -> &[Run] {
		&self.runs
	}

	pub fn tasks(&self) -> &[Task] {
		&self.tasks
	}

	pub fn mm(&self) -> &ModelManager {
		&self.mm
	}

	pub fn last_app_event(&self) -> &LastAppEvent {
		&self.last_app_event
	}
}
