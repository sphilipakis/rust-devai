use super::app_event_handlers::handle_app_event;
use super::event::ActionEvent;
use super::event::{AppEvent, LastAppEvent};
use crate::Result;
use crate::event::Rx;
use crate::exec::ExecutorTx;
use crate::store::ModelManager;
use crate::support::time::now_micro;
use crate::tui::AppState;
use crate::tui::AppTx;
use crate::tui::ExitTx;
use crate::tui::MainView;
use crate::tui::core::app_state::process_app_state;
use crate::tui::core::{PingTimerTx, start_ping_timer};
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

	// Start the ping timer (debouncer) and get its input tx
	let ping_tx: PingTimerTx = start_ping_timer(app_tx.clone())?;

	let handle = tokio::spawn(async move {
		loop {
			// -- Update App State
			process_app_state(&mut app_state);

			// -- If action to send, send it
			if let Some(action_event) = app_state.take_action_event_to_send() {
				let _ = app_tx.send(action_event).await;
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
			let app_event = {
				// -- First we try to see if there one already in the que
				let evt = match app_rx.try_recv() {
					Ok(r) => r,
					Err(err) => {
						error!("UI LOOP ERROR. Cause: {err}");
						continue;
					}
				};

				match evt {
					// No need to do the running tick
					Some(evt) => evt,
					None => {
						// Send a ping event (to the ping_tx debouncer)
						// - running tick
						// - or if we still have a timed popup (to eventually remove it)
						if app_state.running_tick_count().is_some() || app_state.popup().is_some_and(|p| p.is_timed()) {
							let _ = ping_tx.send(now_micro()).await;
						}

						// Then, we wait for next event
						match app_rx.recv().await {
							Ok(evt) => evt,
							Err(err) => {
								error!("UI LOOP ERROR. Cause: {err}");
								continue;
							}
						}
					}
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
