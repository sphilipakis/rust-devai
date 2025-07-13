use crate::Result;
use crate::store::ModelManager;
use crate::store::rt_model::{Run, Task};
use crate::tui::core::sys_state::SysState;
use crate::tui::event::LastAppEvent;

/// The global app state
/// IMPORTANT: We define it in this file so that some state can be private
pub struct AppState {
	// -- Main View
	pub(in crate::tui::core) show_runs: bool,

	// -- RunsView
	pub(in crate::tui::core) run_idx: Option<i32>,

	// -- RunMainView
	// The RunMainView will clamp this one
	// NOTE: probably need to change strategy
	pub run_tab_idx: i32,

	// -- RunDetailsView
	pub(in crate::tui::core) task_idx: Option<i32>,
	pub(in crate::tui::core) before_all_show: bool,
	pub(in crate::tui::core) after_all_show: bool,

	// -- TaskView
	// NOTE: probably need to change strategy
	pub log_scroll: u16, // TaskView will need read/edit

	// -- Data
	// newest to oldest
	pub(in crate::tui::core) runs: Vec<Run>,
	pub(in crate::tui::core) tasks: Vec<Task>,

	// -- System & Event
	pub(in crate::tui::core) mm: ModelManager,
	pub(in crate::tui::core) last_app_event: LastAppEvent,

	// -- SysState
	pub(in crate::tui::core) sys_state: SysState,
	pub(in crate::tui::core) memory: u64,
	pub(in crate::tui::core) cpu: f64,
}

// region:    --- Constructors

impl AppState {
	pub fn new(mm: ModelManager, last_app_event: LastAppEvent) -> Result<Self> {
		let sys_state = SysState::new()?;
		Ok(Self {
			// -- MainView
			show_runs: false,

			// -- RunsView
			run_idx: None,

			// -- RunMainView
			// For now, use the Tasks tab as default
			run_tab_idx: 1,

			// -- RunDetailsView
			task_idx: None,
			log_scroll: 0,
			before_all_show: false,
			after_all_show: false,

			// -- Data
			runs: Vec::new(),
			tasks: Vec::new(),

			// -- System & Event
			mm,
			last_app_event,

			// -- SysState
			sys_state,
			memory: 0,
			cpu: 0.,
		})
	}
}

// endregion: --- Constructors

/// MainView states
impl AppState {
	pub fn show_runs(&self) -> bool {
		self.show_runs
	}
}

/// RunsView states
impl AppState {
	pub fn run_idx(&self) -> Option<usize> {
		self.run_idx.map(|idx| idx as usize)
	}

	pub fn runs(&self) -> &[Run] {
		&self.runs
	}

	pub fn current_run(&self) -> Option<&Run> {
		if let Some(idx) = self.run_idx {
			self.runs.get(idx as usize)
		} else {
			None
		}
	}
}

/// RunDetailsView states
impl AppState {
	pub fn task_idx(&self) -> Option<usize> {
		self.task_idx.map(|idx| idx as usize)
	}

	pub fn tasks(&self) -> &[Task] {
		&self.tasks
	}

	pub fn current_task(&self) -> Option<&Task> {
		if let Some(idx) = self.task_idx {
			self.tasks.get(idx as usize)
		} else {
			None
		}
	}

	/// Returns `true` when the **before-all** pseudo-task is selected.
	pub fn before_all_show(&self) -> bool {
		self.before_all_show
	}

	/// Returns `true` when the **after-all** pseudo-task is selected.
	pub fn after_all_show(&self) -> bool {
		self.after_all_show
	}
}

/// System & Event states
impl AppState {
	pub fn mm(&self) -> &ModelManager {
		&self.mm
	}

	pub fn last_app_event(&self) -> &LastAppEvent {
		&self.last_app_event
	}
}

/// SysState
impl AppState {
	/// This is call in the loop
	pub(in crate::tui::core) fn refresh_sys_state(&mut self) {
		let (memory, cpu) = self.sys_state.memory_and_cpu();
		self.memory = memory;
		self.cpu = cpu;
	}

	/// This is the getters of the refreshed data
	pub fn memory(&self) -> u64 {
		self.memory
	}

	#[allow(unused)]
	pub fn cpu(&self) -> f64 {
		self.cpu
	}
}
