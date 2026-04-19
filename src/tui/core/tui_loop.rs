use super::app_event_handlers::handle_app_event;
use super::event::{AppActionEvent, AppEvent, LastAppEvent};
use crate::Result;
use crate::event::Rx;
use crate::exec::ExecutorTx;
use crate::model::{EntityType, ModelEvent, ModelManager};
use crate::support::time::now_micro;
use crate::tui::core::app_state::{ProcessAppStateOpts, process_app_state};
use crate::tui::core::{PingTimerTx, start_ping_timer};
use crate::tui::{AppState, AppTx, ExitTx, MainView};
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
				// -- First we try to see if there one already in the queue
				// and get the last "refresh_event" if they are stacked
				let mut last_refresh = None;

				let evt = loop {
					match app_rx.try_recv() {
						Ok(Some(r)) => {
							if r.is_refresh_event() {
								last_refresh = Some(r);
								continue;
							} else {
								break Some(r);
							}
						}
						Ok(None) => {
							break last_refresh;
						}
						Err(err) => {
							// NOTE: This might become an infinit loop if the error keep repeating (not sure)
							error!("UI LOOP ERROR.\nCause: {err}");
							continue;
						}
					}
				};

				match evt {
					// No need to do the running tick
					Some(evt) => evt,
					None => {
						// Send a ping event (to the ping_tx debouncer)
						if app_state.should_be_pinged() {
							let _ = ping_tx.send(now_micro()).await;
						}

						// Then, we wait for next event
						match app_rx.recv().await {
							Ok(evt) => evt,
							Err(err) => {
								error!("UI LOOP ERROR.\nCause: {err}");
								continue;
							}
						}
					}
				}
			};

			// NOTE: Handle this specific event here because we need to break the loop
			//       Later, handle_app_event might return a control flow enum
			if let AppEvent::Action(AppActionEvent::Quit) = &app_event {
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

			let process_opts = ProcessAppStateOpts {
				do_refresh_current_tasks: should_refresh_current_tasks(&app_state, &app_event),
			};

			app_state.core_mut().last_app_event = app_event.into();

			// -- Update App State
			process_app_state(&mut app_state, process_opts);

			// -- If action to send, send it
			if let Some(action_event) = app_state.take_action_event_to_send() {
				let _ = app_tx.send(action_event).await;
			}

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

fn should_refresh_current_tasks(app_state: &AppState, app_event: &AppEvent) -> bool {
	if app_state.tasks().is_empty() {
		return true;
	}

	let current_run_id = app_state.current_run_item().map(|run| run.id());
	let loaded_run_id = app_state.run_tasks_info().map(|info| info.run_id());
	if current_run_id != loaded_run_id {
		return true;
	}

	match app_event {
		AppEvent::Data(model_event) => should_refresh_current_tasks_for_model_event(current_run_id, model_event),
		_ => false,
	}
}

fn should_refresh_current_tasks_for_model_event(
	current_run_id: Option<crate::model::Id>,
	model_event: &ModelEvent,
) -> bool {
	match model_event.entity {
		EntityType::Task => match (current_run_id, model_event.rel_ids.run_id) {
			(Some(current_run_id), Some(event_run_id)) => current_run_id == event_run_id,
			(Some(_), None) => true,
			(None, _) => false,
		},
		_ => false,
	}
}
