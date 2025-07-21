use crate::store::rt_model::{RunBmc, TaskBmc};
use crate::tui::AppState;
use crate::tui::core::{MouseEvt, NavDir};
use crate::tui::support::offset_and_clamp_option_idx_in_len;
use crossterm::event::{KeyCode, MouseEventKind};

pub fn process_app_state(state: &mut AppState) {
	// -- Refresh system metrics
	state.refresh_sys_state();

	// -- Capture the mouse Event
	if let Some(mouse_event) = state.last_app_event().as_mouse_event() {
		let mouse_evt: MouseEvt = mouse_event.into();
		state.inner_mut().mouse_evt = Some(mouse_evt);
		// Here we update the persistent mouse
		state.inner_mut().last_mouse_evt = Some(mouse_evt);

		// Find the active scroll zone
		let zone_iden = state.inner().find_zone_for_pos(mouse_evt);

		// if let Some(zone_iden) = zone_iden {
		// 	tracing::debug!(" {zone_iden:?}");
		// }

		state.inner_mut().active_scroll_zone_iden = zone_iden;
	} else {
		state.inner_mut().mouse_evt = None;
		// Note: We do not clear the last_mouse_evt as it should remain persistent
	}

	// -- Scroll
	if let Some(mouse_evt) = state.last_app_event().as_mouse_event()
		&& let Some(zone_iden) = state.inner().active_scroll_zone_iden
	{
		match mouse_evt.kind {
			MouseEventKind::ScrollUp => {
				state.dec_scroll(zone_iden, 1);
			}
			MouseEventKind::ScrollDown => {
				state.inc_scroll(zone_iden, 1);
			}
			_ => (),
		};
	}

	// -- Toggle runs list
	if let Some(KeyCode::Char('n')) = state.last_app_event().as_key_code() {
		let show_runs = !state.inner().show_runs;
		state.inner_mut().show_runs = show_runs;
	}

	// -- Load runs and keep previous idx for later comparison
	let new_runs = RunBmc::list_for_display(state.mm(), None).unwrap_or_default();
	let has_new_runs = new_runs.len() != state.runs().len();
	state.inner_mut().runs = new_runs;

	// only change if we have new runs
	if has_new_runs {
		let prev_run_idx = state.inner().run_idx;
		let prev_run_id = state.inner().run_id;

		{
			let inner = state.inner_mut();

			// When the runs panel is hidden, always pin the latest run (first run index) run.
			if !inner.show_runs {
				inner.set_run_by_idx(0);
			} else {
				// if the prev_run_idx was at 0, then, we keep it at 0
				if prev_run_idx == Some(0) {
					inner.set_run_by_idx(0);
				}
				// otherwise, we preserve the previous id
				else if let Some(prev_run_id) = prev_run_id {
					inner.set_run_by_id(prev_run_id);
				} else {
					inner.set_run_by_idx(0);
				}
			}
		}

		// -- Reset some view state if run selection changed
		// TODO: Need to check if still needed.
		if state.inner().run_idx != prev_run_idx {
			let inner = state.inner_mut();
			inner.task_idx = None;
			inner.before_all_show = false;
			inner.after_all_show = false;
		}
	}

	// -- Navigation inside the runs list
	let runs_nav_offset: i32 = if state.inner().show_runs
		&& let Some(code) = state.last_app_event().as_key_code()
	{
		match code {
			KeyCode::Char('w') => -1,
			KeyCode::Char('s') => 1,
			_ => 0,
		}
	} else {
		0
	};
	if runs_nav_offset != 0 {
		state.inner_mut().offset_run_idx(runs_nav_offset);
	}

	// -- Load tasks for current run
	let current_run_id = state.current_run().map(|r| r.id);
	{
		if let Some(run_id) = current_run_id {
			let tasks = TaskBmc::list_for_run(state.mm(), run_id).unwrap_or_default();
			state.inner_mut().tasks = tasks;
		} else {
			state.inner_mut().tasks.clear(); // Important when no run is selected
		}
	}

	// -- Initialise RunDetailsView if needed
	{
		let need_init = {
			let inner = state.inner();
			inner.task_idx.is_none() && !inner.before_all_show && !inner.after_all_show
		};

		if need_init {
			let tasks_empty = state.tasks().is_empty();
			let inner = state.inner_mut();
			if !tasks_empty {
				inner.task_idx = Some(0);
				inner.before_all_show = false;
				inner.after_all_show = false;
			} else {
				inner.task_idx = None;
				inner.before_all_show = true;
				inner.after_all_show = false;
			}
		}
	}

	// -- Navigation inside the tasks list
	let nav_dir = NavDir::from_up_down_key_code(
		KeyCode::Char('i'),
		KeyCode::Char('k'),
		state.last_app_event().as_key_event(),
	);
	let nav_tasks_offset = nav_dir.map(|n| n.offset()).unwrap_or_default();

	let len_tasks = state.tasks().len();
	{
		let inner = state.inner_mut();
		inner.task_idx = offset_and_clamp_option_idx_in_len(&inner.task_idx, nav_tasks_offset, len_tasks);
	}

	// -- Tabs navigation (Run view)
	if let Some(code) = state.last_app_event().as_key_code() {
		let current_run_tab = state.run_tab();
		match code {
			KeyCode::Char('j') => state.set_run_tab(current_run_tab.prev()),
			KeyCode::Char('l') => state.set_run_tab(current_run_tab.next()),
			_ => (),
		}
	};

	// -- Arrow key (keyboard & mouse)
	// if let Some(code) = state.last_app_event().as_key_code() {
	// 	let log_scroll = match code {
	// 		KeyCode::Up => state.dec_scroll(iden, dec),
	// 		KeyCode::Down => Some(current_log_scroll.saturating_add(1)),
	// 		KeyCode::Esc => Some(0),
	// 		_ => None,
	// 	};
	// 	if let Some(log_scroll) = log_scroll {
	// 		state.set_log_scroll(log_scroll);
	// 	}
	// }

	// -- Debug color
	let offset: i32 = if let Some(code) = state.last_app_event().as_key_code() {
		match code {
			KeyCode::Char('-') => -1,
			KeyCode::Char('=') => 1,
			_ => 0,
		}
	} else {
		0
	};
	match offset {
		-1 => state.dec_debug_clr(),
		1 => state.inc_debug_clr(),
		_ => (),
	}
}
