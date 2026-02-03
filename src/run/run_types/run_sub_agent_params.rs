use crate::Result;
use crate::agent::AgentOptions;
use crate::event::OneShotTx;
use crate::runtime::Runtime;
use crate::types::{RunAgentOptions, RunAgentResponse};
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
		runtime: Runtime,
		parent_uid: Uuid,
		parent_agent_dir: Option<SPath>,
		agent_name: impl Into<String>,
		run_options: RunAgentOptions,
		response_shot: Option<OneShotTx<Result<RunAgentResponse>>>,
	) -> mlua::Result<Self> {
		let RunAgentOptions {
			inputs,
			options: agent_options,
			agent_base_dir,
		} = run_options;

		let agent_dir = agent_base_dir.or(parent_agent_dir);

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
