use crate::Result;
use crate::store::ModelManager;
use crate::store::rt_model::{Run, Task};
use crate::tui::core::sys_state::SysState;
use crate::tui::core::{MouseEvt, RunTab};
use crate::tui::event::LastAppEvent;

// region:    --- Wrapper

/// Public wrapper around [`AppStateInner`].
///
/// Visible only to the `tui` module so it does not leak to the whole crate.
pub struct AppState {
	inner: AppStateInner,
}

// endregion: --- Wrapper

// region:    --- Inner

/// Inner representation of the application state.
///
/// `pub` fields are fine here because the whole struct is only visible
/// inside `crate::tui::core`.
pub(in crate::tui::core) struct AppStateInner {
	// -- Debug
	// debug color idx
	pub debug_clr: u8,

	// -- Mouse
	/// Hold the current app mouse event wrapper (None if last event was not a mouse event)
	pub mouse_evt: Option<MouseEvt>,

	// -- Main View
	pub show_runs: bool,

	// -- RunsView
	pub run_idx: Option<i32>,

	// -- RunMainView
	pub run_tab: RunTab,

	// -- RunTasksView
	pub task_idx: Option<i32>,
	pub before_all_show: bool,
	pub after_all_show: bool,

	// -- TaskView
	pub log_scroll: u16,

	// -- Data
	pub runs: Vec<Run>,
	pub tasks: Vec<Task>,

	// -- System & Event
	pub mm: ModelManager,
	pub do_redraw: bool,
	pub last_app_event: LastAppEvent,

	// -- SysState
	pub sys_state: SysState,
	pub memory: u64,
	pub cpu: f64,
}

// endregion: --- Inner

/// Constructors
impl AppState {
	pub fn new(mm: ModelManager, last_app_event: LastAppEvent) -> Result<Self> {
		let sys_state = SysState::new()?;

		let inner = AppStateInner {
			// -- Debug
			debug_clr: 0,

			// -- Mouse
			mouse_evt: None,

			// -- MainView
			show_runs: false,

			// -- RunsView
			run_idx: None,

			// -- RunMainView
			run_tab: RunTab::Tasks, // Tasks tab by default

			// -- RunTasksView
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
			do_redraw: false,

			// -- SysState
			sys_state,
			memory: 0,
			cpu: 0.,
		};

		Ok(Self { inner })
	}

	// -- Inner accessors

	/// Immutable access to the inner state (core-exclusive).
	pub(in crate::tui::core) fn inner(&self) -> &AppStateInner {
		&self.inner
	}

	/// Mutable access to the inner state (core-exclusive).
	pub(in crate::tui::core) fn inner_mut(&mut self) -> &mut AppStateInner {
		&mut self.inner
	}
}

/// Debug
impl AppState {
	pub fn debug_clr(&self) -> u8 {
		self.inner.debug_clr
	}

	pub(in crate::tui::core) fn inc_debug_clr(&mut self) {
		self.inner.debug_clr = self.inner.debug_clr.wrapping_add(1);
	}

	pub(in crate::tui::core) fn dec_debug_clr(&mut self) {
		self.inner.debug_clr = self.inner.debug_clr.wrapping_sub(1);
	}
}

/// Mouse
impl AppState {
	pub fn mouse_evt(&self) -> Option<MouseEvt> {
		self.inner.mouse_evt
	}
}

/// MainView
impl AppState {
	pub fn show_runs(&self) -> bool {
		self.inner.show_runs
	}
}

/// RunsView
impl AppState {
	pub fn run_idx(&self) -> Option<usize> {
		self.inner.run_idx.map(|idx| idx as usize)
	}

	pub fn set_run_idx(&mut self, idx: Option<usize>) {
		self.inner.run_idx = idx.map(|i| i as i32);
	}

	pub fn runs(&self) -> &[Run] {
		&self.inner.runs
	}

	pub fn current_run(&self) -> Option<&Run> {
		if let Some(idx) = self.inner.run_idx {
			self.inner.runs.get(idx as usize)
		} else {
			None
		}
	}

	pub fn run_tab(&self) -> RunTab {
		self.inner.run_tab
	}

	pub fn set_run_tab(&mut self, run_tab: RunTab) {
		self.inner.run_tab = run_tab;
	}
}

/// RunTasksView
impl AppState {
	pub fn task_idx(&self) -> Option<usize> {
		self.inner.task_idx.map(|idx| idx as usize)
	}

	pub fn set_task_idx(&mut self, idx: Option<usize>) {
		self.inner.task_idx = idx.map(|i| i as i32);
	}

	pub fn tasks(&self) -> &[Task] {
		&self.inner.tasks
	}

	pub fn current_task(&self) -> Option<&Task> {
		if let Some(idx) = self.inner.task_idx {
			self.inner.tasks.get(idx as usize)
		} else {
			None
		}
	}
}

/// System & Event
impl AppState {
	pub fn mm(&self) -> &ModelManager {
		&self.inner.mm
	}

	pub fn last_app_event(&self) -> &LastAppEvent {
		&self.inner.last_app_event
	}
}

/// Other simple accessors
impl AppState {
	pub fn log_scroll(&self) -> u16 {
		self.inner.log_scroll
	}

	pub fn set_log_scroll(&mut self, scroll: u16) {
		self.inner.log_scroll = scroll;
	}

	pub fn should_redraw(&self) -> bool {
		self.inner.do_redraw
	}

	pub fn trigger_redraw(&mut self) {
		self.inner.do_redraw = true;
	}
}

/// SysState & Metrics
impl AppState {
	/// Called every tick of the main loop.
	pub(in crate::tui::core) fn refresh_sys_state(&mut self) {
		let (memory, cpu) = self.inner.sys_state.memory_and_cpu();
		self.inner.memory = memory;
		self.inner.cpu = cpu;
	}

	pub fn memory(&self) -> u64 {
		self.inner.memory
	}

	#[allow(unused)]
	pub fn cpu(&self) -> f64 {
		self.inner.cpu
	}
}
