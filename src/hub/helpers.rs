use crate::Result;
use crate::hub::{Hub, HubEvent};
use crate::tui_v1::PromptParams;

// region:    --- Prompt Via Hub

pub async fn hub_prompt(hub: &Hub, msg: impl Into<String>) -> Result<String> {
	let (params, rx) = PromptParams::new(msg);

	hub.publish(HubEvent::Prompt(params)).await;

	let result = rx.recv().await?;

	Ok(result)
}

// endregion: --- Prompt Via Hub
