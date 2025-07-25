use crate::Result;
use crate::agent::AgentOptions;
use crate::event::OneShotTx;
use crate::run::RunAgentResponse;
use crate::runtime::Runtime;
use crate::script::{LuaValueExt as _, lua_value_to_serde_value};
use mlua::{FromLua, Lua};
use simple_fs::SPath;

/// Options for the agent run function
#[derive(Debug)]
pub struct RunSubAgentParams {
	pub runtime: Runtime,

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
	/// Create an simple RunAgentParams with just the Runtime and agent name / path.
	pub fn new_no_inputs(
		runtime: Runtime,
		agent_dir: Option<SPath>,
		agent_name: impl Into<String>,
		response_shot: Option<OneShotTx<Result<RunAgentResponse>>>,
	) -> Self {
		Self {
			runtime,
			agent_dir,
			agent_name: agent_name.into(),
			response_shot,
			inputs: None,
			agent_options: None,
		}
	}

	pub fn new_from_lua_params(
		runtime: Runtime,
		agent_dir: Option<SPath>,
		agent_name: impl Into<String>,
		response_shot: Option<OneShotTx<Result<RunAgentResponse>>>,
		params: mlua::Value,
		lua: &Lua,
	) -> mlua::Result<Self> {
		let inputs = params.x_get_value("inputs").map(lua_value_to_serde_value).transpose()?;

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
		// Extract the agent options
		let agent_options = params
			.x_get_value("options")
			.map(|o| AgentOptions::from_lua(o, lua))
			.transpose()?;

		Ok(Self {
			runtime,
			agent_dir,
			agent_name: agent_name.into(),
			inputs,
			agent_options,
			response_shot,
		})
	}
}
