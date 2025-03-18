use crate::agent::AgentOptions;
use crate::run::AiResponse;
use crate::script::{LuaValueExt as _, lua_value_to_serde_value};
use mlua::{FromLua, Lua};
use tokio::sync::oneshot;

/// Options for the agent run function
#[derive(Debug, Default)]
pub struct RunAgentParams {
	/// Inputs to pass to the agent
	pub inputs: Option<Vec<serde_json::Value>>,

	/// The eventual agent option overlay
	pub agent_options: Option<AgentOptions>,

	/// The response oneshot with the AiResponse
	#[allow(unused)]
	pub response_shot: Option<oneshot::Sender<AiResponse>>,
}

impl RunAgentParams {
	pub fn new(
		value: mlua::Value,
		lua: &Lua,
		response_shot: Option<oneshot::Sender<AiResponse>>,
	) -> mlua::Result<Self> {
		let inputs = value.x_get_value("inputs").map(lua_value_to_serde_value).transpose()?;

		let inputs = match inputs {
			Some(serde_json::Value::Array(values)) => Some(values),
			None => None,
			_ => {
				return Err(crate::Error::custom(
					"The 'inputs' `aip.agent.run(agent_name, {inputs: ..})` must be a Lua array",
				)
				.into());
			}
		};
		let agent_options = value
			.x_get_value("options")
			.map(|o| AgentOptions::from_lua(o, lua))
			.transpose()?;
		Ok(Self {
			inputs,
			agent_options,
			response_shot,
		})
	}
}
