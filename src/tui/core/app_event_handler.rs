use super::AppTx;
use super::ExitTx;
use crate::Result;
use crate::exec::{ExecActionEvent, ExecutorTx};
use crate::hub::HubEvent;
use crate::store::ModelManager;
use crate::store::rt_model::{LogBmc, LogForCreate, LogKind};
use crate::tui::event::{ActionEvent, AppEvent, DataEvent};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

// region:    --- Public

pub async fn handle_app_event(
	terminal: &mut DefaultTerminal,
	mm: &ModelManager,
	executor_tx: &ExecutorTx,
	app_tx: &AppTx,
	exit_tx: &ExitTx,
	app_event: &AppEvent,
) -> Result<()> {
	// tracing::debug!("APP EVENT HANDLER - {app_event:?}");

	match app_event {
		AppEvent::Term(term_event) => {
			handle_term_event(term_event, app_tx).await?;
		}
		AppEvent::Action(action_event) => {
			handle_action_event(action_event, terminal, executor_tx, exit_tx).await?;
		}
		AppEvent::Data(data_event) => {
			handle_data_event(data_event).await?;
		}
		AppEvent::Hub(hub_event) => {
			handle_hub_event(mm, hub_event).await?;
		}
	};

	Ok(())
}

// endregion: --- Public

// region:    --- Handlers

async fn handle_term_event(term_event: &Event, app_tx: &AppTx) -> Result<()> {
	if let Event::Key(key) = term_event {
		if let KeyEventKind::Press = key.kind {
			let mod_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
			match (key.code, mod_ctrl) {
				(KeyCode::Char('q'), _) | (KeyCode::Char('c'), true) => app_tx.send(ActionEvent::Quit).await?,
				(KeyCode::Char('r'), _) => app_tx.send(ActionEvent::Redo).await?,
				_ => (),
			}
		}
	}
	Ok(())
}

async fn handle_action_event(
	action_event: &ActionEvent,
	_terminal: &mut DefaultTerminal,
	executor_tx: &ExecutorTx,
	_exit_tx: &ExitTx,
) -> Result<()> {
	match action_event {
		// -- The quick is handle at the main loop to break
		ActionEvent::Quit => {
			// Handled at the main loop
		}

		// -- Do the Redo
		ActionEvent::Redo => {
			//
			executor_tx.send(ExecActionEvent::Redo).await;
		}
	}
	Ok(())
}

async fn handle_data_event(data_event: &DataEvent) -> Result<()> {
	println!("DataEvent {data_event:?}");
	Ok(())
}

#[allow(clippy::single_match)]
async fn handle_hub_event(mm: &ModelManager, hub_event: &HubEvent) -> Result<()> {
	match hub_event {
		// -- Message
		HubEvent::Message(_msg) => {
			// FIXME: need to have an Error table or someting
			// LogBmc::create(
			// 	mm,
			// 	LogForCreate {
			// 		run_id: 0.into(),
			// 		task_id: None,
			// 		level: Some(LogLevel::SysInfo),
			// 		step: None,
			// 		stage: None,
			// 		message: Some(msg.to_string()),
			// 	},
			// )?;
		}

		// -- Handle Lua Action
		// NOTE: for now, just the LuaPrint, but we will generialize it
		HubEvent::LuaPrint(_msg) => {
			// NOW, we do this in the aip lua print directly otherwise out of order with other log events, like skip
		}

		// -- Error
		HubEvent::Error { error } => {
			// FIXME: need to have an Error table or someting
			//        Here hack on run = 0
			LogBmc::create(
				mm,
				LogForCreate {
					run_id: 0.into(),
					task_id: None,
					kind: Some(LogKind::SysError),
					step: None,
					stage: None,
					message: Some(error.to_string()),
				},
			)?;
		}
		_ => (),
	};

	Ok(())
}

// endregion: --- Handlers
