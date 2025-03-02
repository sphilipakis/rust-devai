// region:    --- AiResponse

use crate::support::W;
use genai::ModelName;
use genai::adapter::AdapterKind;
use genai::chat::MetaUsage;
use mlua::IntoLua;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AiResponse {
	pub content: Option<String>,
	pub reasoning_content: Option<String>,
	pub model_name: ModelName,
	pub adapter_kind: AdapterKind,
	pub usage: MetaUsage,
	pub price_usd: Option<f64>,
	pub duration_sec: f64,
	pub info: String,
}

impl IntoLua for AiResponse {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;

		table.set("content", self.content.into_lua(lua)?)?;
		table.set("reasoning_content", self.reasoning_content.into_lua(lua)?)?;
		table.set("model_name", self.model_name.into_lua(lua)?)?;
		table.set("adapter_kind", self.adapter_kind.as_str().into_lua(lua)?)?;
		table.set("usage", W(&self.usage).into_lua(lua)?)?;
		table.set("price_usd", self.price_usd.into_lua(lua)?)?;
		table.set("duration_sec", self.duration_sec.into_lua(lua)?)?;
		table.set("info", self.info.into_lua(lua)?)?;

		Ok(mlua::Value::Table(table))
	}
}

impl IntoLua for W<&MetaUsage> {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		let usage = self.0;

		table.set("prompt_tokens", usage.prompt_tokens.into_lua(lua)?)?;
		table.set("completion_tokens", usage.completion_tokens.into_lua(lua)?)?;

		// -- Prompt Details
		// Note: we create the details even if None (simpler on the script side)
		let prompt_details_table = lua.create_table()?;
		if let Some(prompt_tokens_details) = usage.prompt_tokens_details.as_ref() {
			// Note: The leaf value can be absent (same as nil in Lua)
			if let Some(v) = prompt_tokens_details.cached_tokens {
				prompt_details_table.set("cached_tokens", v.into_lua(lua)?)?;
			}
			if let Some(v) = prompt_tokens_details.audio_tokens {
				prompt_details_table.set("audio_tokens", v.into_lua(lua)?)?;
			}
		}
		table.set("prompt_tokens_details", prompt_details_table)?;

		// -- Completion Details
		// Note: we create the details even if None (simpler on the script side)
		let completion_details_table = lua.create_table()?;
		if let Some(completion_tokens_details) = usage.completion_tokens_details.as_ref() {
			// Note: The leaf value can be absent (same as nil in Lua)
			if let Some(v) = completion_tokens_details.reasoning_tokens {
				completion_details_table.set("reasoning_tokens", v.into_lua(lua)?)?;
			}
			if let Some(v) = completion_tokens_details.audio_tokens {
				completion_details_table.set("audio_tokens", v.into_lua(lua)?)?;
			}
			if let Some(v) = completion_tokens_details.accepted_prediction_tokens {
				completion_details_table.set("accepted_prediction_tokens", v.into_lua(lua)?)?;
			}
			if let Some(v) = completion_tokens_details.rejected_prediction_tokens {
				completion_details_table.set("rejected_prediction_tokens", v.into_lua(lua)?)?;
			}
		}
		table.set("completion_tokens_details", completion_details_table)?;

		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- AiResponse
