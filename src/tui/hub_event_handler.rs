use crate::Result;
use crate::exec::{ExecActionEvent, ExecStatusEvent, ExecutorSender};
use crate::hub::HubEvent;
use crate::tui::prompter::prompt;
use crate::tui::support::safer_println;
use crate::tui::{handle_print, tui_elem};

pub async fn handle_hub_event(event: HubEvent, exec_sender: &ExecutorSender, interactive: bool) -> Result<()> {
	match event {
		HubEvent::Message(msg) => {
			safer_println(&format!("{msg}"), interactive);
		}

		HubEvent::Error { error } => {
			safer_println(&format!("Error: {error}"), interactive);
		}

		HubEvent::LuaPrint(text) => safer_println(&text, interactive),

		HubEvent::Print(print_event) => handle_print(print_event, interactive),

		// HubEvent::Prompt(params) => prompt(params).await?,
		HubEvent::Executor(exec_event) => {
			if let (ExecStatusEvent::RunEnd, true) = (exec_event, interactive) {
				tui_elem::print_bottom_bar();
			}
		}
		HubEvent::DoExecRedo => exec_sender.send(ExecActionEvent::Redo).await,
		HubEvent::Quit => {
			// Nothing to do for now
		}
	}

	Ok(())
}
