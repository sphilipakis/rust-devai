use crate::Result;
use crate::exec::{ExecCommand, ExecEvent};
use crate::hub::HubEvent;
use crate::tui::support::{safer_println, send_to_executor};
use crate::tui::tui_elem;
use tokio::sync::mpsc;

pub async fn handle_hub_event(event: HubEvent, exec_tx: &mpsc::Sender<ExecCommand>, interactive: bool) -> Result<()> {
	match event {
		HubEvent::Message(msg) => {
			safer_println(&format!("{msg}"), interactive);
		}
		HubEvent::Error { error } => {
			safer_println(&format!("Error: {error}"), interactive);
		}

		HubEvent::LuaPrint(text) => safer_println(&text, interactive),

		HubEvent::Executor(exec_event) => {
			if let (ExecEvent::RunEnd, true) = (exec_event, interactive) {
				// safer_println("\n[ r ]: Redo   |   [ q ]: Quit", interactive);
				tui_elem::print_bottom_bar();
			}
		}
		HubEvent::DoExecRedo => send_to_executor(exec_tx, ExecCommand::Redo).await,
		HubEvent::Quit => {
			// Nothing to do for now
		}
	}

	Ok(())
}
