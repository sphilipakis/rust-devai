use crate::Result;
use crate::agent::Agent;
use crate::run::AiResponse;
use crate::run::literals::Literals;
use crate::run::proc_ai::{ProcAiResponse, process_ai};
use crate::run::proc_data::{ProcDataResponse, process_data};
use crate::run::{DryMode, RunBaseOptions};
use crate::runtime::Runtime;
use crate::store::rt_model::RuntimeCtx;
use crate::store::{Id, Stage};
use serde_json::Value;

// region:    --- RunAgentInputResponse

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum RunAgentInputResponse {
	AiReponse(AiResponse),
	OutputResponse(Value),
}

impl RunAgentInputResponse {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			RunAgentInputResponse::AiReponse(ai_response) => ai_response.content.as_deref(),
			RunAgentInputResponse::OutputResponse(value) => value.as_str(),
		}
	}

	/// Note: for now, we do like this. Might want to change that.
	/// Note: There is something to do about AI being able to structured output and manage it her
	/// - If AiResposne take the String as value or Null
	/// - If OutputResponse, then, the value is result
	pub fn into_value(self) -> Value {
		match self {
			RunAgentInputResponse::AiReponse(ai_response) => ai_response.content.into(),
			RunAgentInputResponse::OutputResponse(value) => value,
		}
	}
}

// endregion: --- RunAgentInputResponse

/// Run the agent for one input
/// - Build the scope
/// - Execute Data
/// - Render the prompt sections
/// - Send the AI
/// - Execute Output
///
/// Note 1: For now, this will create a new Lua engine.
///         This is likely to stay as it creates a strong segregation between input execution
#[allow(clippy::too_many_arguments)]
pub async fn run_agent_task(
	runtime: &Runtime,
	run_id: Id,
	task_id: Id,
	agent: &Agent,
	before_all_result: Value,
	_label: &str,
	input: Value,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<Option<RunAgentInputResponse>> {
	let client = runtime.genai_client();

	// -- Build Base Rt Context
	let base_rt_ctx = RuntimeCtx::from_run_task_ids(runtime, Some(run_id), Some(task_id))?;

	// -- Process Data Stage
	// Rt Step - Start Data stage
	runtime.step_task_data_start(run_id, task_id).await?;
	let res = process_data(
		runtime,
		base_rt_ctx.clone(),
		run_id,
		task_id,
		agent.clone(),
		literals,
		&before_all_result,
		input,
	)
	.await;
	// Capture error if any
	if let Err(err) = res.as_ref() {
		runtime.set_task_end_error(run_id, task_id, Some(Stage::Data), err)?;
	}
	// Rt Step - End Data stage
	runtime.step_task_data_end(run_id, task_id).await?;

	let ProcDataResponse {
		agent,
		input,
		data,
		run_model_resolved,
		skip,
	} = res?;
	if skip {
		return Ok(None);
	}

	// -- Execute genai if we have an instruction
	// Rt Step - Start AI stage
	runtime.step_task_ai_start(run_id, task_id).await?;
	let res = process_ai(
		runtime,
		client,
		run_base_options,
		&run_model_resolved,
		run_id,
		task_id,
		agent.clone(),
		&before_all_result,
		&input,
		&data,
	)
	.await;
	// Capture error if any
	if let Err(err) = res.as_ref() {
		runtime.set_task_end_error(run_id, task_id, Some(Stage::Ai), err)?;
	}
	let ProcAiResponse { skip, ai_response } = res?;
	if skip {
		return Ok(None);
	}

	// -- if dry_mode res, we stop
	if matches!(run_base_options.dry_mode(), DryMode::Res) {
		return Ok(None);
	}

	// -- Exec output

	let res = if let Some(output_script) = agent.output_script() {
		// -- Rt Step - start output
		runtime.step_task_output_start(run_id, task_id).await?;

		// -- Create the Output Lua Engine
		let lua_engine = runtime.new_lua_engine_with_ctx(literals, base_rt_ctx.with_stage(Stage::Output))?;

		// -- Create the scope
		let lua_scope = lua_engine.create_table()?;
		lua_scope.set("input", lua_engine.serde_to_lua_value(input)?)?;
		lua_scope.set("data", lua_engine.serde_to_lua_value(data)?)?;
		lua_scope.set("before_all", lua_engine.serde_to_lua_value(before_all_result)?)?;
		lua_scope.set("ai_response", ai_response)?;
		lua_scope.set("options", agent.options_as_ref())?;

		let lua_value = lua_engine.eval(output_script, Some(lua_scope), Some(&[agent.file_dir()?.as_str()]))?;
		let output_response = serde_json::to_value(lua_value)?;

		// -- Rt Step - end output
		runtime.step_task_output_end(run_id, task_id).await?;

		Some(RunAgentInputResponse::OutputResponse(output_response))
	} else {
		ai_response.map(RunAgentInputResponse::AiReponse)
	};

	Ok(res)
}
