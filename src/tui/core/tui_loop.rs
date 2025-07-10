use crate::Result;
use crate::event::Rx;
use crate::exec::ExecutorTx;
use crate::store::ModelManager;
use crate::tui::AppState;
use crate::tui::AppTx;
use crate::tui::ExitTx;
use crate::tui::MainView;
use crate::tui::app_event_handler::handle_app_event;
use crate::tui::core::state_processor::process_app_state;
use crate::tui::event::ActionEvent;
use crate::tui::event::{AppEvent, LastAppEvent};
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
			app_state.last_app_event = app_event.into();
		}
	});
	Ok(handle)
}

fn terminal_draw(terminal: &mut DefaultTerminal, app_state: &mut AppState) -> Result<()> {
	terminal.draw(|frame| {
		let area = frame.area();

		let main_view = MainView {};
		frame.render_stateful_widget(main_view, area, app_state);
	})?;

	Ok(())
}
