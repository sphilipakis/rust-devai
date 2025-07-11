use crate::store::rt_model::RunBmc;
use crate::store::rt_model::TaskBmc;
use crate::tui::AppState;
use crate::tui::core::NavDir;
use crate::tui::support::offset_and_clamp_option_idx_in_len;
use crossterm::event::{KeyCode, MouseEventKind};

pub fn process_app_state(state: &mut AppState) {
	state.refresh_sys_state();

	// -- Process Show run
	if let Some(KeyCode::Char('n')) = state.last_app_event().as_key_code() {
		state.show_runs = !state.show_runs
	}

	// -- load runs
	let runs_limit = if state.show_runs { None } else { Some(1) };
	let runs = RunBmc::list_for_display(state.mm(), runs_limit).unwrap_or_default();
	let prev_run_idx = state.run_idx; // to compute scroll status
	// Make sure to select the first one (now thete there is only ones
	if !state.show_runs {
		state.run_idx = Some(0);
	}
	state.runs = runs;

	// -- Process Runs idx
	let runs_nav_offset: i32 = if let Some(code) = state.last_app_event().as_key_code() {
		match code {
			KeyCode::Char('w') => -1,
			KeyCode::Char('s') => 1,
			_ => 0,
		}
	} else {
		0
	};

	// -- Clamp the run_idx with the runs_lent
	state.run_idx = offset_and_clamp_option_idx_in_len(&state.run_idx, runs_nav_offset, state.runs().len());

	// -- load tasks for current run
	let current_run_id = state.current_run().map(|r| r.id);
	if let Some(run_id) = current_run_id {
		state.tasks = TaskBmc::list_for_run(state.mm(), run_id).unwrap_or_default();
	} else {
		state.tasks.clear(); // Important to clear tasks if no run is selected
	}

	// -- if run changed, reset log scroll and task selection
	if state.run_idx != prev_run_idx {
		state.log_scroll = 0;
		// if there are tasks for the new run, select the first one
		state.task_idx = if !state.tasks.is_empty() { Some(0) } else { None };
	}

	// -- Process Run Task idx with 'i' and 'k'

	let nav_dir = NavDir::from_up_down_key_code(
		KeyCode::Char('i'),
		KeyCode::Char('k'),
		state.last_app_event().as_key_event(),
	);

	// let offset: i32 = nav_dir.map(|v| v.offset()).unwrap_or_default();

	let tasks_max_idx = state.tasks().len() as i32 - 1;
	// only do someting if we have an offset
	if let Some(nav_dir) = nav_dir {
		// -- Navigate or get out of tasks nav
		if let Some(tab_idx) = state.task_idx {
			let new_idx = tab_idx + nav_dir.offset();
			// we show before all
			if new_idx < 0 {
				state.task_idx = None;
				state.before_all_show = true;
				state.after_all_show = false;
			}
			// we show after all
			else if new_idx > tasks_max_idx {
				state.task_idx = None;
				state.before_all_show = false;
				state.after_all_show = true;
			} else {
				state.task_idx = Some(new_idx)
			}
		}
		// -- Enter task from top
		else if nav_dir.is_down() && state.before_all_show {
			state.before_all_show = false;
			state.task_idx = Some(0);
		}
		// -- Enter in tasks nav from bottom
		else if nav_dir.is_up() && state.after_all_show {
			state.after_all_show = false;
			state.task_idx = Some(tasks_max_idx);
		}
	}

	//state.task_idx = offset_and_clamp_option_idx_in_len(&state.task_idx, offset, tasks.len());

	// -- process the Run Tabs idx
	let offset: i32 = if let Some(code) = state.last_app_event().as_key_code() {
		match code {
			KeyCode::Char('j') => -1,
			KeyCode::Char('l') => 1,
			_ => 0,
		}
	} else {
		0
	};
	state.run_tab_idx += offset;

	// -- Process log scroll (keyboard & mouse)
	if let Some(code) = state.last_app_event().as_key_code() {
		match code {
			KeyCode::Up => state.log_scroll = state.log_scroll.saturating_sub(1),
			KeyCode::Down => state.log_scroll = state.log_scroll.saturating_add(1),
			KeyCode::Esc => state.log_scroll = 0,
			_ => (),
		}
	}

	if let Some(mouse_evt) = state.last_app_event().as_mouse_event() {
		match mouse_evt.kind {
			MouseEventKind::ScrollUp => {
				state.log_scroll = state.log_scroll.saturating_sub(3);
			}
			MouseEventKind::ScrollDown => {
				state.log_scroll = state.log_scroll.saturating_add(3);
			}
			_ => (),
		}
	}
}
