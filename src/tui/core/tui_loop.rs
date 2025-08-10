use super::app_event_handlers::handle_app_event;
use super::event::ActionEvent;
use super::event::{AppEvent, LastAppEvent};
use crate::Result;
use crate::event::Rx;
use crate::exec::ExecutorTx;
use crate::store::ModelManager;
use crate::tui::AppState;
use crate::tui::AppTx;
use crate::tui::ExitTx;
use crate::tui::MainView;
use crate::tui::core::app_state::process_app_state;
use crate::tui::core::{Action, RunTab};
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
	// Initialize App State (fail early, in case of SysState fail to initialize)
	let mut app_state = AppState::new(mm, LastAppEvent::default())?;

	let handle = tokio::spawn(async move {
		loop {
			// -- Update App State
			process_app_state(&mut app_state);

			// -- Do the action
			// Just show the right tab/view
			if let Some(action) = app_state.action() {
				match action {
					Action::GoToTask { .. } => {
						app_state.set_run_tab(RunTab::Tasks);
					}
				}
				// -- trigger a redraw
				// NOTE: Now that we do not fully exec the action here, we cannot trigger a redraw here
				// let _ = app_tx.send(AppEvent::DoRedraw).await;
			}

			// -- Draw
			let _ = terminal_draw(&mut terminal, &mut app_state);

			// -- Trigger the redraw if needed
			// TODO: We might want to have a timestamp so that we do not process the redraw
			//       if another event happened before.
			if app_state.should_redraw() {
				app_state.core_mut().do_redraw = false;
				let _ = app_tx.send(AppEvent::DoRedraw).await;
			}

			// -- Get Next App Event
			let app_event = match app_rx.recv().await {
				Ok(r) => r,
				Err(err) => {
					error!("UI LOOP ERROR. Cause: {err}");
					continue;
				}
			};

			// NOTE: Handle this specific event here because we need to break the loop
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
			app_state.core_mut().last_app_event = app_event.into();
		}
	});
	Ok(handle)
}

fn terminal_draw(terminal: &mut DefaultTerminal, app_state: &mut AppState) -> Result<()> {
	terminal.draw(|frame| {
		let area = frame.area();

		frame.render_stateful_widget(MainView, area, app_state);
	})?;

	Ok(())
}
