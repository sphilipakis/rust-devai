use super::AppStateCore;
use crate::Result;
use crate::store::ModelManager;
use crate::store::rt_model::{Run, Task};
use crate::tui::core::event::LastAppEvent;
use crate::tui::core::sys_state::SysState;
use crate::tui::core::{Action, RunTab, ScrollZones};

// region:    --- Wrapper

/// Public wrapper around AppStateCor.
///
/// Visible only to the `tui` module so it does not leak to the whole crate.
pub struct AppState {
	pub(in crate::tui::core) core: AppStateCore,
}

// endregion: --- Wrapper

/// Constructors
impl AppState {
	pub fn new(mm: ModelManager, last_app_event: LastAppEvent) -> Result<Self> {
		let sys_state = SysState::new()?;

		let inner = AppStateCore {
			// -- Debug
			debug_clr: 0,

			// -- Mouse
			mouse_evt: None,
			last_mouse_evt: None,

			// -- ScrollZones
			scroll_zones: ScrollZones::default(),
			active_scroll_zone_iden: None,

			// -- MainView
			show_runs: false,

			// -- RunsView
			run_idx: None,
			run_id: None,

			// -- RunMainView
			run_tab: RunTab::Tasks, // Tasks tab by default

			// -- RunTasksView
			task_idx: None,
			before_all_show: false,
			after_all_show: false,

			// -- Data
			runs: Vec::new(),
			tasks: Vec::new(),

			// -- System & Event
			mm,
			last_app_event,

			// -- Action
			do_redraw: false,
			do_action: None,

			// -- SysState
			sys_state,
			memory: 0,
			cpu: 0.,
		};

		Ok(Self { core: inner })
	}

	// -- Inner accessors

	/// Immutable access to the inner state (core-exclusive).
	pub(in crate::tui::core) fn core(&self) -> &AppStateCore {
		&self.core
	}

	/// Mutable access to the inner state (core-exclusive).
	pub(in crate::tui::core) fn core_mut(&mut self) -> &mut AppStateCore {
		&mut self.core
	}
}

/// Debug
impl AppState {
	pub fn debug_clr(&self) -> u8 {
		self.core.debug_clr
	}

	pub(in crate::tui::core) fn inc_debug_clr(&mut self) {
		self.core.debug_clr = self.core.debug_clr.wrapping_add(1);
	}

	pub(in crate::tui::core) fn dec_debug_clr(&mut self) {
		self.core.debug_clr = self.core.debug_clr.wrapping_sub(1);
	}
}

/// MainView
impl AppState {
	pub fn show_runs(&self) -> bool {
		self.core.show_runs
	}
}

/// RunsView
impl AppState {
	pub fn run_idx(&self) -> Option<usize> {
		self.core.run_idx.map(|idx| idx as usize)
	}

	pub fn set_run_idx(&mut self, idx: Option<usize>) {
		if let Some(idx) = idx {
			self.core.set_run_by_idx(idx as i32);
		}
	}

	pub fn runs(&self) -> &[Run] {
		&self.core.runs
	}

	pub fn current_run(&self) -> Option<&Run> {
		if let Some(idx) = self.core.run_idx {
			self.core.runs.get(idx as usize)
		} else {
			None
		}
	}

	pub fn run_tab(&self) -> RunTab {
		self.core.run_tab
	}

	pub fn set_run_tab(&mut self, run_tab: RunTab) {
		self.core.run_tab = run_tab;
	}
}

/// RunTasksView
impl AppState {
	pub fn task_idx(&self) -> Option<usize> {
		self.core.task_idx.map(|idx| idx as usize)
	}

	pub fn set_task_idx(&mut self, idx: Option<usize>) {
		self.core.task_idx = idx.map(|i| i as i32);
	}

	pub fn tasks(&self) -> &[Task] {
		&self.core.tasks
	}

	pub fn current_task(&self) -> Option<&Task> {
		if let Some(idx) = self.core.task_idx {
			self.core.tasks.get(idx as usize)
		} else {
			None
		}
	}
}

/// System & Event
impl AppState {
	pub fn mm(&self) -> &ModelManager {
		&self.core.mm
	}

	pub fn last_app_event(&self) -> &LastAppEvent {
		&self.core.last_app_event
	}
}

impl AppState {
	pub fn set_action(&mut self, action: impl Into<Action>) {
		let action = action.into();
		self.core.do_action = Some(action);
	}
}

/// Others
impl AppState {
	pub fn should_redraw(&self) -> bool {
		self.core.do_redraw
	}

	pub fn trigger_redraw(&mut self) {
		self.core.do_redraw = true;
	}
}

/// SysState & Metrics
impl AppState {
	/// Called every tick of the main loop.
	pub(in crate::tui::core) fn refresh_sys_state(&mut self) {
		let (memory, cpu) = self.core.sys_state.memory_and_cpu();
		self.core.memory = memory;
		self.core.cpu = cpu;
	}

	pub fn memory(&self) -> u64 {
		self.core.memory
	}

	#[allow(unused)]
	pub fn cpu(&self) -> f64 {
		self.core.cpu
	}
}
