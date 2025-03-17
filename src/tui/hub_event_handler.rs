use crate::Result;
use crate::exec::{ExecActionEvent, ExecStatusEvent};
use crate::hub::HubEvent;
use crate::tui::support::{safer_println, send_to_executor};
use crate::tui::{handle_print, tui_elem};
use tokio::sync::mpsc;

pub async fn handle_hub_event(
	event: HubEvent,
	exec_tx: &mpsc::Sender<ExecActionEvent>,
	interactive: bool,
) -> Result<()> {
	match event {
		HubEvent::Message(msg) => {
			safer_println(&format!("{msg}"), interactive);
		}

		HubEvent::Error { error } => {
			safer_println(&format!("Error: {error}"), interactive);
		}

		HubEvent::LuaPrint(text) => safer_println(&text, interactive),

		HubEvent::Print(print_event) => handle_print(print_event, interactive),

		HubEvent::Executor(exec_event) => {
			if let (ExecStatusEvent::RunEnd, true) = (exec_event, interactive) {
				tui_elem::print_bottom_bar();
			}
		}
		HubEvent::DoExecRedo => send_to_executor(exec_tx, ExecActionEvent::Redo).await,
		HubEvent::Quit => {
			// Nothing to do for now
		}
	}

	Ok(())
}
