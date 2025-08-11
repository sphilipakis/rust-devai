use crate::store::rt_model::{ErrBmc, RunBmc, TaskBmc};
use crate::support::time::now_micro;
use crate::tui::AppState;
use crate::tui::core::{Action, MouseEvt, NavDir, RunItemStore, RunTab};
use crate::tui::support::offset_and_clamp_option_idx_in_len;
use crate::tui::view::{PopupMode, PopupView};
use crossterm::event::{KeyCode, MouseEventKind};
use std::time::Duration;

pub fn process_app_state(state: &mut AppState) {
	// -- Process tick
	state.core.time = now_micro();

	// -- Process actions (clipboard, show-text popup, tab switch)
	process_actions(state);

	// -- Expire timed popups
	if let Some(PopupMode::Timed(duration)) = state.popup().map(|p| &p.mode)
		&& let Some(start) = state.core().popup_start_us
		&& state.core().time.saturating_sub(start) >= duration.as_micros() as i64
	{
		state.clear_popup();
	}

	// -- Dismiss user popups on Esc
	if let Some(key_event) = state.last_app_event().as_key_event()
		&& key_event.code == KeyCode::Esc
		&& let Some(popup) = state.popup()
		&& matches!(popup.mode, PopupMode::User)
	{
		state.clear_popup();
	}

	// -- Toggle show sys state
	if let Some(key_event) = state.last_app_event().as_key_event()
		&& key_event.code == KeyCode::Char('M')
		&& key_event.modifiers.contains(crossterm::event::KeyModifiers::SHIFT)
	{
		state.toggle_show_sys_states();
	}

	// -- Refresh system metrics
	if state.show_sys_states() {
		state.refresh_sys_state();
	}

	// -- Capture the mouse Event
	if let Some(mouse_event) = state.last_app_event().as_mouse_event() {
		let mouse_evt: MouseEvt = mouse_event.into();
		state.core_mut().mouse_evt = Some(mouse_evt);
		// Here we update the persistent mouse
		state.core_mut().last_mouse_evt = Some(mouse_evt);

		// Find the active scroll zone
		let zone_iden = state.core().find_zone_for_pos(mouse_evt);

		// if let Some(zone_iden) = zone_iden {
		// 	tracing::debug!(" {zone_iden:?}");
		// }

		state.core_mut().active_scroll_zone_iden = zone_iden;
	} else {
		state.core_mut().mouse_evt = None;
		// Note: We do not clear the last_mouse_evt as it should remain persistent
	}

	// -- Scroll
	if let Some(mouse_evt) = state.last_app_event().as_mouse_event()
		&& let Some(zone_iden) = state.core().active_scroll_zone_iden
	{
		match mouse_evt.kind {
			MouseEventKind::ScrollUp => {
				state.core_mut().dec_scroll(zone_iden, 1);
			}
			MouseEventKind::ScrollDown => {
				state.core_mut().inc_scroll(zone_iden, 1);
			}
			_ => (),
		};
	}

	// -- Toggle runs list
	if let Some(KeyCode::Char('n')) = state.last_app_event().as_key_code() {
		let show_runs = !state.core().show_runs;
		state.core_mut().show_runs = show_runs;
	}

	// -- Cycle tasks overview mode
	if let Some(KeyCode::Char('t')) = state.last_app_event().as_key_code() {
		state.core_mut().next_overview_tasks_mode();
	}

	// -- Load runs and keep previous idx for later comparison
	let new_runs = RunBmc::list_for_display(state.mm(), None).unwrap_or_default();
	let runs_len = new_runs.len();
	let has_new_runs = new_runs.len() != state.run_items().len();
	let run_item_store = RunItemStore::new(new_runs);
	state.core_mut().run_item_store = run_item_store;

	// only change if we have new runs
	if has_new_runs {
		let prev_run_idx = state.core().run_idx;
		let prev_run_id = state.core().run_id;

		{
			let inner = state.core_mut();

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
		if state.core().run_idx != prev_run_idx {
			let inner = state.core_mut();
			inner.task_idx = None;
		}
	}

	// -- Fetch System Error
	// NOTE: For now, we will assume that system errors are before the first run
	// TODO: Eventually, this might not be true, as user could break the config.toml.
	if runs_len == 0 {
		// For now, ignore potential infra erro
		if let Ok(sys_err) = ErrBmc::first_system_err(state.mm()) {
			state.core_mut().sys_err = sys_err
		}
	}

	// -- Navigation inside the runs list
	let runs_nav_offset: i32 = if state.core().show_runs
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
		state.core_mut().offset_run_idx(runs_nav_offset);
	}

	// -- Load tasks for current run
	let current_run_id = state.current_run_item().map(|r| r.id());
	{
		if let Some(run_id) = current_run_id {
			let tasks = TaskBmc::list_for_run(state.mm(), run_id).unwrap_or_default();
			let tasks_len = tasks.len();
			state.core_mut().tasks = tasks;
			// Important to avoid the "no current task" where there is ne.
			// Need to reset task_idx to 0 if current task_idx is > that tasks
			if let Some(current_task_idx) = state.core().task_idx
				&& current_task_idx > tasks_len as i32 - 1
			{
				state.set_task_idx(Some(0));
			}
		} else {
			state.core_mut().tasks.clear(); // Important when no run is selected
		}
	}

	// -- Initialise RunDetailsView if needed
	{
		let need_init = { state.core().task_idx.is_none() };

		if need_init {
			let tasks_empty = state.tasks().is_empty();
			let inner = state.core_mut();
			if !tasks_empty {
				inner.task_idx = Some(0);
			} else {
				inner.task_idx = None;
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

	if nav_tasks_offset != 0 {
		let len_tasks = state.tasks().len();
		let inner = state.core();
		let new_task_idx =
			offset_and_clamp_option_idx_in_len(&inner.task_idx, nav_tasks_offset, len_tasks).unwrap_or_default();
		if let Some(task) = state.tasks().get(new_task_idx as usize) {
			state.set_action(Action::GoToTask { task_id: task.id });
			// Note: Little trick to not show the hover when navigating
			state.clear_mouse_evts();
		}
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

	// -- Update running tick
	let run_items = state.run_items();
	if !run_items.is_empty() {
		let is_one_running = run_items.iter().any(|r| r.is_running());

		// If running and no running start, then, set the running_start
		if is_one_running && state.core.running_tick_start.is_none() {
			state.core.running_tick_start = Some(now_micro())
		}
		// Make sure to turn it off if not running
		else if !is_one_running {
			state.core.running_tick_start = None
		}
	} else {
		state.core.running_tick_start = None; // No run selected
	}

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

// region:    --- Action Processing

fn process_actions(state: &mut AppState) {
	if let Some(action) = state.action().cloned() {
		match action {
			Action::ToClipboardCopy(content) => {
				// Ensure we have a clipboard instance
				let ensure_clipboard: Result<(), String> = if state.core().clipboard.is_some() {
					Ok(())
				} else {
					match arboard::Clipboard::new() {
						Ok(cb) => {
							state.core_mut().clipboard = Some(cb);
							Ok(())
						}
						Err(err) => Err(format!("Clipboard init error: {err}")),
					}
				};

				let popup_msg = match ensure_clipboard {
					Ok(()) => {
						if let Some(cb) = state.core_mut().clipboard.as_mut() {
							match cb.set_text(content) {
								Ok(()) => "Copied to clipboard".to_string(),
								Err(err) => format!("Clipboard error: {err}"),
							}
						} else {
							"Clipboard unavailable".to_string()
						}
					}
					Err(msg) => msg,
				};

				state.set_popup(PopupView {
					content: popup_msg,
					mode: PopupMode::Timed(Duration::from_millis(1000)),
				});
				state.clear_action();
			}
			Action::ShowText => {
				state.set_popup(PopupView {
					content: "Click on Content".to_string(),
					mode: PopupMode::Timed(Duration::from_millis(1000)),
				});
				state.clear_action();
			}
			Action::GoToTask { .. } => {
				// Switch to Tasks tab; keep the action so the view can select and clear it.
				state.set_run_tab(RunTab::Tasks);
			}
		}
	}
}

// endregion: --- Action Processing
