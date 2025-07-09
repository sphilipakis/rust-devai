use crate::Result;
use crate::event::Rx;
use crate::exec::ExecutorTx;
use crate::store::ModelManager;
use crate::store::rt_model::RunBmc;
use crate::store::rt_model::TaskBmc;
use crate::tui::AppState;
use crate::tui::AppTx;
use crate::tui::ExitTx;
use crate::tui::MainView;
use crate::tui::app_event_handler::handle_app_event;
use crate::tui::event::ActionEvent;
use crate::tui::event::{AppEvent, LastAppEvent};
use crate::tui::support::offset_and_clamp_option_idx_in_len;
use crossterm::event::{KeyCode, MouseEventKind};
use ratatui::DefaultTerminal;
use tokio::task::JoinHandle;
use tracing::error;

pub fn run_ui_loop(
	mut terminal: DefaultTerminal,
	mm: ModelManager,
	executor_tx: ExecutorTx,
	app_rx: Rx<AppEvent>,
	app_tx: AppTx,
	exit_tx: ExitTx,
) -> Result<JoinHandle<()>> {
	let handle = tokio::spawn(async move {
		// Initialize App State
		let mut app_state = AppState::new(mm, LastAppEvent::default());

		loop {
			// -- Update App State
			process_app_state(&mut app_state);

			// -- Draw
			let _ = terminal_draw(&mut terminal, &mut app_state);

			// -- Get Next App Event
			let app_event = match app_rx.recv().await {
				Ok(r) => r,
				Err(err) => {
					error!("UI LOOP ERROR. Cause: {err}");
					continue;
				}
			};

			// NOTE: Handle this specific even there because we need to break the llop
			//       Later, handle_app_event might return a control flow enum
			if let AppEvent::Action(ActionEvent::Quit) = &app_event {
				let _ = terminal.clear();
				let _ = exit_tx.send(()).await;
				break;
			}

			let _ = handle_app_event(
				&mut terminal,
				app_state.mm(),
				&executor_tx,
				&app_tx,
				&exit_tx,
				&app_event,
			)
			.await;

			// Update the last_app_event
			app_state.last_app_event = app_event.into();
		}
	});
	Ok(handle)
}

fn process_app_state(state: &mut AppState) {
	// -- load runs
	let runs = RunBmc::list_for_display(state.mm()).unwrap_or_default();
	let prev_run_idx = state.run_idx;
	state.runs = runs;

	// -- Process Runs idx
	let offset: i32 = if let Some(code) = state.last_app_event().as_key_code() {
		match code {
			KeyCode::Char('w') => -1,
			KeyCode::Char('s') => 1,
			_ => 0,
		}
	} else {
		0
	};

	// -- Clamp the run_idx with the runs_lent
	state.run_idx = offset_and_clamp_option_idx_in_len(&state.run_idx, offset, state.runs().len());

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

	// -- Process Task idx with 'i' and 'k'
	let offset: i32 = if let Some(code) = state.last_app_event().as_key_code() {
		match code {
			KeyCode::Char('i') => -1,
			KeyCode::Char('k') => 1,
			_ => 0,
		}
	} else {
		0
	};
	state.task_idx = offset_and_clamp_option_idx_in_len(&state.task_idx, offset, state.tasks().len());

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

fn terminal_draw(terminal: &mut DefaultTerminal, app_state: &mut AppState) -> Result<()> {
	terminal.draw(|frame| {
		let area = frame.area();

		let main_view = MainView {};
		frame.render_stateful_widget(main_view, area, app_state);
	})?;

	Ok(())
}
