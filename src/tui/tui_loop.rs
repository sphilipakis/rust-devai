use super::event::{AppEvent, LastAppEvent};
use crate::Result;
use crate::event::Rx;
use crate::exec::ExecutorTx;
use crate::store::ModelManager;
use crate::store::rt_model::Run;
use crate::store::rt_model::RunBmc;
use crate::tui::AppTx;
use crate::tui::ExitTx;
use crate::tui::MainView;
use crate::tui::app_event_handler::handle_app_event;
use crate::tui::event::ActionEvent;
use ratatui::DefaultTerminal;
use tokio::task::JoinHandle;
use tracing::error;

/// The global app state
/// IMPORTANT: We define it in this file so that some state can be private
pub struct AppState {
	pub run_idx: Option<i32>,

	// newest to oldest
	runs: Vec<Run>,

	mm: ModelManager,
	last_app_event: LastAppEvent,
}

impl AppState {
	pub fn new(mm: ModelManager, last_app_event: LastAppEvent) -> Self {
		Self {
			run_idx: None,
			runs: Vec::new(),
			mm,
			last_app_event,
		}
	}
	pub fn current_run(&self) -> Option<&Run> {
		if let Some(idx) = self.run_idx {
			self.runs.get(idx as usize)
		} else {
			None
		}
	}

	pub fn runs(&self) -> &[Run] {
		&self.runs
	}

	pub fn mm(&self) -> &ModelManager {
		&self.mm
	}

	pub fn last_app_event(&self) -> &LastAppEvent {
		&self.last_app_event
	}
}

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
			let runs = RunBmc::list_for_display(app_state.mm()).unwrap_or_default();
			app_state.runs = runs;

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
			//debug!("->> run_ui_loop AppEvent: {app_event:?}");

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

fn terminal_draw(terminal: &mut DefaultTerminal, app_state: &mut AppState) -> Result<()> {
	terminal.draw(|frame| {
		let area = frame.area();

		let main_view = MainView {};
		frame.render_stateful_widget(main_view, area, app_state);
	})?;

	Ok(())
}
