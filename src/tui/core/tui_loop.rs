use super::app_event_handlers::handle_app_event;
use super::event::{AppActionEvent, AppEvent, LastAppEvent};
use crate::Result;
use crate::exec::ExecutorTx;
use crate::hub::HubEvent;
use crate::model::{EntityType, Id, ModelManager};
use crate::support::time::now_micro;
use crate::tui::core::app_state::{ProcessAppStateOpts, process_app_state};
use crate::tui::core::tui_impl::AppRx;
use crate::tui::core::{PingTimerTx, start_ping_timer};
use crate::tui::{AppState, AppTx, ExitTx, MainView};
use crossterm::event::Event as TermEvent;
use ratatui::DefaultTerminal;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use tracing::error;

pub fn run_ui_loop(
	mut terminal: DefaultTerminal,
	mm: ModelManager,
	executor_tx: ExecutorTx,
	mut app_rx: AppRx,
	app_tx: AppTx,
	exit_tx: ExitTx,
) -> Result<JoinHandle<()>> {
	// Initialize App State (fail early, in case of SysState fail to initialize)
	let mut app_state = AppState::new(mm, LastAppEvent::default())?;

	// Start the ping timer (debouncer) and get its input tx
	let ping_tx: PingTimerTx = start_ping_timer(app_tx.clone())?;

	let handle = tokio::spawn(async move {
		'outer: loop {
			// This allows to not have an infinit loop and async way
			let app_event = match app_rx.recv().await {
				Ok(app_event) => app_event,
				Err(err) => {
					error!("Fail to app_rx.recv().await  in tui_loop. Cause: {err}");
					break;
				}
			};
			// -- Debounce the events (with this first_event and the eventual ones in the list)
			let (new_app_rx, events) = debounce_events(app_rx, app_event);
			app_rx = new_app_rx;

			if events.is_empty() && app_state.should_be_pinged() {
				let _ = ping_tx.send(now_micro()).await;
			}

			for app_event in events {
				if let AppEvent::Term(TermEvent::Mouse(mouse_event)) = &app_event {
					app_state.set_mouse_event(mouse_event);
				}
				// -- Draw
				let _ = terminal_draw(&mut terminal, &mut app_state);

				// -- HANDLE Quit
				// NOTE: Handle this specific event here because we need to break the loop
				//       Later, handle_app_event might return a control flow enum
				if let AppEvent::Action(AppActionEvent::Quit) = &app_event {
					let _ = terminal.clear();
					let _ = exit_tx.send(()).await;
					break 'outer;
				}

				// -- Normal handle
				let _ = handle_app_event(
					&mut terminal,
					app_state.mm(),
					&executor_tx,
					&app_tx,
					&exit_tx,
					&app_event,
				)
				.await;

				// -- Process app sate
				let process_opts = ProcessAppStateOpts {
					current_event_refreshes_tasks: current_event_refreshes_tasks(&app_event),
				};

				app_state.core_mut().last_app_event = app_event.into();

				process_app_state(&mut app_state, process_opts);

				// -- If action to send, send it
				if let Some(action_event) = app_state.take_action_event_to_send() {
					let _ = app_tx.send(action_event).await;
				}

				// -- Trigger the redraw if needed
				// TODO: We might want to have a timestamp so that we do not process the redraw
				//       if another event happened before.
				if app_state.should_redraw() {
					app_state.core_mut().do_redraw = false;
					let _ = app_tx.send(AppEvent::DoRedraw).await;
				}
			}
		}
	});
	Ok(handle)
}

struct Debouncer {
	last_redraw_event: Option<AppEvent>,
	ui_events: Vec<AppEvent>,
	event_by_run_id: HashMap<Id, AppEvent>,
}

impl Debouncer {
	fn new(first_event: AppEvent) -> Self {
		let mut debouncer = Self {
			last_redraw_event: None,
			ui_events: Vec::new(),
			event_by_run_id: HashMap::new(),
		};
		debouncer.process(first_event);
		debouncer
	}

	fn process(&mut self, app_event: AppEvent) {
		match app_event {
			AppEvent::DoRedraw => {
				self.last_redraw_event = {
					//
					Some(AppEvent::DoRedraw)
				}
			}
			AppEvent::Term(event) => {
				//
				self.ui_events.push(AppEvent::Term(event))
			}
			AppEvent::Action(action_event) => {
				//
				self.ui_events.push(AppEvent::Action(action_event))
			}
			AppEvent::Model(model_event) | AppEvent::Hub(HubEvent::Model(model_event)) => {
				let for_run_id = match &model_event.entity {
					EntityType::Task if let Some(run_id) = model_event.rel_ids.run_id => Some(run_id),
					EntityType::Run if let Some(run_id) = model_event.id => Some(run_id),
					_ => None,
				};

				// add back the app event as appropriate
				let app_event = AppEvent::Model(model_event);
				if let Some(run_id) = for_run_id {
					self.event_by_run_id.insert(run_id, app_event);
				} else {
					self.last_redraw_event = Some(app_event);
				}
			}
			AppEvent::Hub(hub_event) => self.last_redraw_event = Some(AppEvent::Hub(hub_event)),
			AppEvent::Tick(tick) => self.last_redraw_event = Some(AppEvent::Tick(tick)),
		}
	}

	fn into_events(self) -> Vec<AppEvent> {
		let mut events = self.ui_events;
		events.extend(self.event_by_run_id.into_values());
		// for now, append the last redraw (might not be needed)
		if let Some(last_redraw_event) = self.last_redraw_event {
			events.push(last_redraw_event);
		}
		events
	}
}

/// Here we draing the AppRx AppEvents with a debounced version
/// There is couple of strategies
/// - DoRedraw, will get ignored if other event in the list
/// - HubEvent(Model(entity = Task)) for the same run get collapse two the latest one
///     - Will be added at the event of the list
/// - HubEvent(other) will be ingored (or latest)
/// -
fn debounce_events(app_rx: AppRx, first_event: AppEvent) -> (AppRx, Vec<AppEvent>) {
	let mut debouncer = Debouncer::new(first_event);
	loop {
		match app_rx.try_recv() {
			Ok(Some(app_event)) => {
				debouncer.process(app_event);
			}

			Ok(None) => break,

			// TODO: need to log an error or something
			Err(_) => break,
		}
	}

	let events = debouncer.into_events();

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
