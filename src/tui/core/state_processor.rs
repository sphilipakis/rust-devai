use crate::store::rt_model::{RunBmc, TaskBmc};
use crate::tui::AppState;
use crate::tui::core::NavDir;
use crate::tui::support::offset_and_clamp_option_idx_in_len;
use crossterm::event::{KeyCode, MouseEventKind};
use tracing::debug;

pub fn process_app_state(state: &mut AppState) {
	// -- Refresh system metrics
	state.refresh_sys_state();

	// -- Toggle runs list
	if let Some(KeyCode::Char('n')) = state.last_app_event().as_key_code() {
		let show_runs = !state.inner().show_runs;
		state.inner_mut().show_runs = show_runs;
	}

	// -- Load runs and keep previous idx for later comparison
	let runs = RunBmc::list_for_display(state.mm(), None).unwrap_or_default();
	let prev_run_idx = state.inner().run_idx;

	{
		let inner = state.inner_mut();

		// When the runs panel is hidden, always pin the first run.
		if !inner.show_runs {
			inner.run_idx = Some(0);
		}

		inner.runs = runs;
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

	let len_runs = state.runs().len();
	{
		let inner = state.inner_mut();
		inner.run_idx = offset_and_clamp_option_idx_in_len(&inner.run_idx, runs_nav_offset, len_runs);
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

	// -- Reset some view state if run selection changed
	if state.inner().run_idx != prev_run_idx {
		let inner = state.inner_mut();
		inner.log_scroll = 0;
		inner.task_idx = None;
		inner.before_all_show = false;
		inner.after_all_show = false;
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
	let offset: i32 = if let Some(code) = state.last_app_event().as_key_code() {
		match code {
			KeyCode::Char('j') => -1,
			KeyCode::Char('l') => 1,
			_ => 0,
		}
	} else {
		0
	};
	state.inner_mut().run_tab_idx += offset;

	// -- Log scroll (keyboard & mouse)
	let current_log_scroll = state.log_scroll();
	if let Some(code) = state.last_app_event().as_key_code() {
		let log_scroll = match code {
			KeyCode::Up => Some(current_log_scroll.saturating_sub(1)),
			KeyCode::Down => Some(current_log_scroll.saturating_add(1)),
			KeyCode::Esc => Some(0),
			_ => None,
		};
		if let Some(log_scroll) = log_scroll {
			state.set_log_scroll(log_scroll);
		}
	}

	let current_log_scroll = state.log_scroll();
	if let Some(mouse_evt) = state.last_app_event().as_mouse_event() {
		let log_scroll = match mouse_evt.kind {
			MouseEventKind::ScrollUp => Some(current_log_scroll.saturating_sub(3)),
			MouseEventKind::ScrollDown => Some(current_log_scroll.saturating_add(3)),
			_ => None,
		};
		debug!("!!! mouse event .. {log_scroll:?}");
		if let Some(log_scroll) = log_scroll {
			state.set_log_scroll(log_scroll);
		}
	}
}
