use crate::exec::{ExecActionEvent, ExecStatusEvent, ExecutorSender};
use crate::hub::HubEvent;
use crate::tui::prompter::prompt;
use crate::tui::support::safer_println;
use crate::tui::{PrintEvent, handle_print, tui_elem};
use crate::{Error, Result};

pub async fn handle_hub_event(event: HubEvent, exec_sender: &ExecutorSender, interactive: bool) -> Result<()> {
	match event {
		HubEvent::Message(msg) => {
			safer_println(&format!("{msg}"), interactive);
		}

		HubEvent::Error { error } => match &*error {
			Error::GenAIEnvKeyMissing { model_iden, env_name } => handle_print(
				PrintEvent::ApiKeyEnvMissing {
					model_iden: model_iden.clone(),
					env_name: env_name.to_string(),
				}
				.into(),
				interactive,
			),
			other => safer_println(&format!("\nError: {other}"), interactive),
		},

		HubEvent::LuaPrint(text) => safer_println(&text, interactive),

		HubEvent::Print(print_event) => handle_print(print_event, interactive),

		HubEvent::Prompt(params) => prompt(&params).await?,

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
