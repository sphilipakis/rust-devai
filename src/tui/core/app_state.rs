use crate::Result;
use crate::store::rt_model::{Run, Task};
use crate::store::{Id, ModelManager};
use crate::tui::core::sys_state::SysState;
use crate::tui::core::{MouseEvt, RunTab, ScrollIden, ScrollZone, ScrollZones};
use crate::tui::event::LastAppEvent;
use crate::tui::support::offset_and_clamp_option_idx_in_len;
use ratatui::layout::{Position, Rect};

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
	/// This keep the current/last mouse event
	pub last_mouse_evt: Option<MouseEvt>,

	// -- Scroll Zones
	pub scroll_zones: ScrollZones,
	pub active_scroll_zone_iden: Option<ScrollIden>,

	// -- Main View
	pub show_runs: bool,

	// -- RunsView
	pub run_idx: Option<i32>,
	pub run_id: Option<Id>,

	// -- RunMainView
	pub run_tab: RunTab,

	// -- RunTasksView
	pub task_idx: Option<i32>,
	pub before_all_show: bool,
	pub after_all_show: bool,

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

impl AppStateInner {
	pub fn set_run_by_idx(&mut self, idx: i32) {
		self.run_idx = Some(idx);
		self.run_id = self.runs.get(idx as usize).map(|r| r.id);
	}

	pub fn set_run_by_id(&mut self, run_id: Id) {
		let run_idx = self.runs.iter().position(|r| r.id == run_id);
		self.run_idx = run_idx.map(|v| v as i32);
		// For now, we set it a None if not found (need to revise strategy, can syncup with the by_idx)
		self.run_id = run_idx.map(|_| run_id);
	}

	pub fn offset_run_idx(&mut self, offset: i32) {
		let runs_len = self.runs.len();
		let new_idx = offset_and_clamp_option_idx_in_len(&self.run_idx, offset, runs_len);
		if let Some(new_idx) = new_idx {
			self.set_run_by_idx(new_idx);
		}
	}
}

/// Scroll Inner impl
impl AppStateInner {
	pub fn find_zone_for_pos(&self, position: impl Into<Position>) -> Option<ScrollIden> {
		self.scroll_zones.find_zone_for_pos(position)
	}
	#[allow(unused)]
	pub fn get_zone(&self, iden: &ScrollIden) -> Option<&ScrollZone> {
		self.scroll_zones.zones.get(iden)
	}

	pub fn get_zone_mut(&mut self, iden: &ScrollIden) -> Option<&mut ScrollZone> {
		self.scroll_zones.zones.get_mut(iden)
	}

	pub fn get_scroll(&self, iden: ScrollIden) -> u16 {
		self.scroll_zones.zones.get(&iden).and_then(|z| z.scroll()).unwrap_or_default()
	}

	/// return the new value
	pub fn inc_scroll(&mut self, iden: ScrollIden, inc: u16) -> u16 {
		let val = self.get_scroll(iden);
		let val = val.saturating_add(inc);
		if let Some(z) = self.get_zone_mut(&iden) {
			z.set_scroll(val);
		}

		val
	}

	pub fn dec_scroll(&mut self, iden: ScrollIden, dec: u16) -> u16 {
		let val = self.get_scroll(iden);
		let val = val.saturating_sub(dec);
		if let Some(z) = self.get_zone_mut(&iden) {
			z.set_scroll(val);
		}

		val
	}
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

/// Scroll
impl AppState {
	pub fn set_scroll_area(&mut self, iden: ScrollIden, area: Rect) {
		if let Some(zone) = self.inner.get_zone_mut(&iden) {
			zone.set_area(area);
		}
	}

	pub fn clear_scroll_zone_area(&mut self, iden: &ScrollIden) {
		if let Some(zone) = self.inner.get_zone_mut(iden) {
			zone.clear_area();
		}
	}

	pub fn clear_scroll_zone_areas(&mut self, idens: &[&ScrollIden]) {
		for iden in idens {
			self.clear_scroll_zone_area(iden);
		}
	}

	/// Note: will return 0 if no scroll was set yet
	#[allow(unused)]
	pub fn get_scroll(&self, iden: ScrollIden) -> u16 {
		self.inner.get_scroll(iden)
	}

	pub fn clamp_scroll(&mut self, iden: ScrollIden, line_count: usize) -> u16 {
		let Some(scroll_zone) = self.inner.get_zone_mut(&iden) else {
			return 0;
		};
		let area_height = scroll_zone.area().map(|a| a.height).unwrap_or_default();
		let max_scroll = line_count.saturating_sub(area_height as usize) as u16;
		let scroll = scroll_zone.scroll().unwrap_or_default();
		if scroll > max_scroll {
			scroll_zone.set_scroll(max_scroll);
			max_scroll
		} else {
			scroll
		}
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
		if let Some(idx) = idx {
			self.inner.set_run_by_idx(idx as i32);
		}
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

/// Others
impl AppState {
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
