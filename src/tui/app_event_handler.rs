use crate::Result;
use crate::hub::HubEvent;
use crate::store::ModelManager;
use crate::store::rt_model::{LogBmc, LogForCreate, LogKind};
use crate::tui::ExitTx;
use crate::tui::event::AppEvent;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use tracing::debug;

pub async fn handle_app_event(
	terminal: &mut DefaultTerminal,
	mm: &ModelManager,
	exit_tx: &ExitTx,
	app_event: &AppEvent,
) -> Result<()> {
	let _ = mm;
	debug!("APP EVENT HANDLER - {app_event:?}");
	match &app_event {
		// -- Term event
		AppEvent::Term(crossterm::event::Event::Key(key)) => {
			// -- Handle Quit Event
			if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q')
				|| key.kind == KeyEventKind::Press
					&& key.modifiers.contains(KeyModifiers::CONTROL)
					&& key.code == KeyCode::Char('c')
			{
				terminal.clear();
				exit_tx.send(()).await;
			}
			// -- TODO: More key Event
		}

		AppEvent::Data(data_event) => {
			println!("DataEvent {data_event:?}")
		}

		AppEvent::Hub(HubEvent::Error { error }) => {
			// FIXME: need to have an Error table or someting
			//        Her,e hack on run = 0
			LogBmc::create(
				mm,
				LogForCreate {
					run_id: 0.into(),
					task_id: None,
					kind: Some(LogKind::SysError),
					stage: None,
					message: Some(error.to_string()),
				},
			)?;
		}
		_ => (),
	};

	Ok(())
}
