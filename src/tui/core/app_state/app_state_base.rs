use super::AppStateCore;
use crate::Result;
use crate::store::ModelManager;
use crate::store::rt_model::Task;
use crate::tui::core::event::LastAppEvent;
use crate::tui::core::sys_state::SysState;
use crate::tui::core::{OverviewTasksMode, RunItem, RunTab, ScrollZones};

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

			// -- RunOverview
			overview_tasks_mode: OverviewTasksMode::Auto,

			// -- RunTasksView
			task_idx: None,

			// -- Data
			run_items: Vec::new(),
			tasks: Vec::new(),

			// -- System & Event
			mm,
			last_app_event,

			// -- Action
			do_redraw: false,
			do_action: None,

			// -- SysState
			show_sys_states: false,
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

	pub fn run_items(&self) -> &[RunItem] {
		&self.core.run_items
	}

	pub fn current_run_item(&self) -> Option<&RunItem> {
		if let Some(idx) = self.core.run_idx {
			self.core.run_items.get(idx as usize)
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

/// OverviewView
impl AppState {
	pub fn overview_tasks_mode(&self) -> OverviewTasksMode {
		self.core.overview_tasks_mode
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
	/// Called every tick of the main loop (if show_sys_states)
	pub(in crate::tui::core) fn refresh_sys_state(&mut self) {
		let (memory, cpu) = self.core.sys_state.memory_and_cpu();
		self.core.memory = memory;
		self.core.cpu = cpu;
	}

	/// Called from the app state processor on Shift M
	pub(in crate::tui::core) fn toggle_show_sys_states(&mut self) {
		self.core.show_sys_states = !self.core.show_sys_states;
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
