use crate::store::ModelManager;
use crate::store::rt_model::{Run, Task};
use crate::tui::event::LastAppEvent;

/// The global app state
/// IMPORTANT: We define it in this file so that some state can be private
pub struct AppState {
	pub(in crate::tui) run_idx: Option<i32>,
	pub(in crate::tui) task_idx: Option<i32>,

	// -- RunView
	pub(in crate::tui) log_scroll: u16,
	pub run_tab_idx: i32,

	// -- Data
	// newest to oldest
	pub(in crate::tui) runs: Vec<Run>,
	pub(in crate::tui) tasks: Vec<Task>,

	// -- System & Event
	pub(in crate::tui) mm: ModelManager,
	pub(in crate::tui) last_app_event: LastAppEvent,
}

/// Contrustor
impl AppState {
	pub fn new(mm: ModelManager, last_app_event: LastAppEvent) -> Self {
		Self {
			run_idx: None,
			task_idx: None,

			// -- RunView
			log_scroll: 0,
			run_tab_idx: 0,

			// Data
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

	pub fn task_idx(&self) -> Option<usize> {
		self.task_idx.map(|idx| idx as usize)
	}

	pub fn current_task(&self) -> Option<&Task> {
		if let Some(idx) = self.task_idx {
			self.tasks.get(idx as usize)
		} else {
			None
		}
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
