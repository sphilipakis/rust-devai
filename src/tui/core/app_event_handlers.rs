use super::AppTx;
use super::ExitTx;
use super::event::{ActionEvent, AppEvent, DataEvent, ScrollDir};
use crate::Result;
use crate::exec::{ExecActionEvent, ExecutorTx};
use crate::hub::HubEvent;
use crate::model::ModelManager;
use crate::model::{LogBmc, LogForCreate, LogKind};
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
	// if let AppEvent::Term(Event::Mouse(mouse_event)) = app_event {
	// 	tracing::debug!("TUI Mouse AppEvent: {mouse_event:?}");
	// }
	// tracing::debug!("APP EVENT HANDLER - {app_event:?}");

	match app_event {
		AppEvent::DoRedraw => (),  // nothing special, this will trigger a redraw
		AppEvent::Tick(_ts) => (), // nothing, just will do a refresh if needed

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

/// Briddge a term event (e.g., keyboard) into an Action Event
async fn handle_term_event(term_event: &Event, app_tx: &AppTx) -> Result<()> {
	if let Event::Key(key) = term_event
		&& let KeyEventKind::Press = key.kind
	{
		let mod_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
		let mod_shift = key.modifiers.contains(KeyModifiers::SHIFT);

		//if matches!(key.code, KeyCode::Up | KeyCode::Down) {
		// tracing::debug!(
		// 	"{:?} TUI Key Event: code: {:?}, ctrl: {mod_ctrl}, shift: {mod_shift}",
		// 	key,
		// 	key.code
		// );
		// }

		match (key.code, mod_ctrl, mod_shift) {
			(KeyCode::Char('q'), _, _) | (KeyCode::Char('c'), true, _) => app_tx.send(ActionEvent::Quit).await?,
			(KeyCode::Char('r'), _, _) => app_tx.send(ActionEvent::Redo).await?,
			(KeyCode::Char('x'), _, _) => app_tx.send(ActionEvent::CancelRun).await?,

			// -- Scroll To End (Shift + Arrow or Home/End)
			(KeyCode::Up, _, true) | (KeyCode::Home, _, _) => {
				app_tx.send(ActionEvent::ScrollToEnd(ScrollDir::Up)).await?
			}
			(KeyCode::Down, _, true) | (KeyCode::End, _, _) => {
				app_tx.send(ActionEvent::ScrollToEnd(ScrollDir::Down)).await?
			}

			// -- Scroll Page (PageUp/PageDown)
			(KeyCode::PageUp, _, _) => app_tx.send(ActionEvent::ScrollPage(ScrollDir::Up)).await?,
			(KeyCode::PageDown, _, _) => app_tx.send(ActionEvent::ScrollPage(ScrollDir::Down)).await?,

			// -- Scroll (Arrow)
			(KeyCode::Up, _, false) => app_tx.send(ActionEvent::Scroll(ScrollDir::Up)).await?,
			(KeyCode::Down, _, false) => app_tx.send(ActionEvent::Scroll(ScrollDir::Down)).await?,

			_ => (),
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
		ActionEvent::Quit => {
			// Handled at the main loop
		}
		ActionEvent::Redo => {
			//
			executor_tx.send(ExecActionEvent::Redo).await;
		}
		ActionEvent::CancelRun => {
			//
			executor_tx.send(ExecActionEvent::CancelRun).await;
		}
		ActionEvent::Scroll(_) => (),
		ActionEvent::ScrollPage(_) => (),
		ActionEvent::ScrollToEnd(_) => (),
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
