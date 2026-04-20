use super::app_event_handlers::handle_app_event;
use super::event::{AppActionEvent, AppEvent, LastAppEvent};
use crate::Result;
use crate::event::Rx;
use crate::exec::ExecutorTx;
use crate::hub::HubEvent;
use crate::model::{EntityType, Id, ModelManager};
use crate::support::time::now_micro;
use crate::tui::core::app_state::{ProcessAppStateOpts, process_app_state};
use crate::tui::core::tui_impl::AppRx;
use crate::tui::core::{PingTimerTx, start_ping_timer};
use crate::tui::{AppState, AppTx, ExitTx, MainView};
use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};
use tokio::task::JoinHandle;
use tracing::error;

pub fn run_ui_loop(
	mut terminal: DefaultTerminal,
	mm: ModelManager,
	executor_tx: ExecutorTx,
	app_rx: AppRx,
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
				current_event_refreshes_tasks: current_event_refreshes_tasks(&app_event),
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

/// Here we draing the AppRx AppEvents with a debounced version
/// There is couple of strategies
/// - DoRedraw, will get ignored if other event in the list
/// - HubEvent(Model(entity = Task)) for the same run get collapse two the latest one
///     - Will be added at the event of the list
/// - HubEvent(other) will be ingored (or latest)
/// -
fn debounce_events(app_rx: AppRx) -> (AppRx, Vec<AppEvent>) {
	let mut last_redraw_event: Option<AppEvent> = None;
	let mut ui_events = Vec::new();

	let mut event_by_run_id: HashMap<Id, AppEvent> = HashMap::new();

	loop {
		match app_rx.try_recv() {
			Ok(Some(app_event)) => {
				//
				match app_event {
					AppEvent::DoRedraw => last_redraw_event = Some(AppEvent::DoRedraw),
					AppEvent::Term(event) => ui_events.push(AppEvent::Term(event)),
					AppEvent::Action(action_event) => ui_events.push(AppEvent::Action(action_event)),
					AppEvent::Model(model_event) | AppEvent::Hub(HubEvent::Model(model_event)) => {
						let for_run_id = match &model_event.entity {
							EntityType::Task if let Some(run_id) = model_event.rel_ids.run_id => Some(run_id),
							EntityType::Run if let Some(run_id) = model_event.id => Some(run_id),
							_ => None,
						};

						// add back the app event as appropriate
						let app_event = AppEvent::Model(model_event);
						if let Some(run_id) = for_run_id {
							event_by_run_id.insert(run_id, app_event);
						} else {
							last_redraw_event = Some(app_event);
						}
					}
					AppEvent::Hub(hub_event) => last_redraw_event = Some(AppEvent::Hub(hub_event)),
					AppEvent::Tick(tick) => last_redraw_event = Some(AppEvent::Tick(tick)),
				}
			}

			Ok(None) => break,

			// TODO: need to log an error or something
			Err(_) => break,
		}
	}

	//
	let mut events = ui_events;
	events.extend(event_by_run_id.into_values());
	// for now, append the last redraw (might not be needed)
	if let Some(last_redraw_event) = last_redraw_event {
		events.push(last_redraw_event);
	}

	(app_rx, events)
}

fn terminal_draw(terminal: &mut DefaultTerminal, app_state: &mut AppState) -> Result<()> {
	terminal.draw(|frame| {
		let area = frame.area();

		frame.render_stateful_widget(MainView, area, app_state);
	})?;

	Ok(())
}

fn current_event_refreshes_tasks(app_event: &AppEvent) -> bool {
	let model_event = match app_event {
		AppEvent::Model(model_event) => Some(model_event),
		AppEvent::Hub(HubEvent::Model(model_event)) => Some(model_event),
		_ => None,
	};

	if let Some(model_event) = model_event {
		matches!(model_event.entity, EntityType::Task | EntityType::Run)
	} else {
		false
	}
}
