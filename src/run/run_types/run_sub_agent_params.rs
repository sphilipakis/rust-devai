use crate::Result;
use crate::agent::AgentOptions;
use crate::event::OneShotTx;
use crate::runtime::Runtime;
use crate::script::{LuaValueExt as _, lua_value_to_serde_value};
use crate::types::RunAgentResponse;
use mlua::{FromLua, Lua};
use simple_fs::SPath;
use uuid::Uuid;

/// Options for the agent run function
#[derive(Debug)]
pub struct RunSubAgentParams {
	pub runtime: Runtime,

	pub parent_uid: Uuid,

	pub agent_dir: Option<SPath>,

	pub agent_name: String,

	/// Inputs to pass to the agent
	pub inputs: Option<Vec<serde_json::Value>>,

	/// The eventual agent option overlay
	pub agent_options: Option<AgentOptions>,

	/// The response oneshot with the RunAgentResponse
	pub response_shot: Option<OneShotTx<Result<RunAgentResponse>>>,
}

impl RunSubAgentParams {
	pub fn new(
		lua: &Lua,
		runtime: Runtime,
		parent_uid: Uuid,
		agent_dir: Option<SPath>,
		agent_name: impl Into<String>,
		response_shot: Option<OneShotTx<Result<RunAgentResponse>>>,
		params: Option<mlua::Value>,
	) -> mlua::Result<Self> {
		let (inputs, agent_options) = if let Some(params) = params {
			let inputs = params.x_get_value("inputs").map(lua_value_to_serde_value).transpose()?;
			let agent_options = params
				.x_get_value("options")
				.map(|o| AgentOptions::from_lua(o, lua))
				.transpose()?;
			(inputs, agent_options)
		} else {
			(None, None)
		};

		// extract the inputs
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

		Ok(Self {
			runtime,
			parent_uid,
			agent_dir,
			agent_name: agent_name.into(),
			inputs,
			agent_options,
			response_shot,
		})
	}
}
