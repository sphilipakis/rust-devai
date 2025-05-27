use crate::agent::{Agent, AgentOptions, AgentRef};
use crate::dir_context::DirContext;
use crate::hub::{HubEvent, get_hub};
use crate::run::RunBaseOptions;
use crate::run::literals::Literals;
use crate::run::run_input::{RunAgentInputResponse, run_agent_input};
use crate::runtime::Runtime;
use crate::script::{AipackCustom, BeforeAllResponse, FromValue, serde_value_to_lua_value, serde_values_to_lua_values};
use crate::{Error, Result};
use mlua::IntoLua;
use serde::Serialize;
use serde_json::Value;
use simple_fs::SPath;
use tokio::task::JoinSet;
use value_ext::JsonValueExt;

const DEFAULT_CONCURRENCY: usize = 1;

pub async fn run_command_agent(
	runtime: &Runtime,
	agent: Agent,
	inputs: Option<Vec<Value>>,
	run_base_options: &RunBaseOptions,
	return_output_values: bool,
) -> Result<RunAgentResponse> {
	let hub = get_hub();

	let literals = Literals::from_runtime_and_agent_path(runtime, &agent)?;

	// display relative agent path if possible
	let agent_path = match get_display_path(agent.file_path(), runtime.dir_context()) {
		Ok(path) => path.to_string(),
		Err(_) => agent.file_path().to_string(),
	};

	hub.publish(format!(
		"\n======= RUNNING: {}\n     Agent path: {agent_path}\n",
		agent.name()
	))
	.await;

	// -- Run the before all
	let BeforeAllResponse {
		inputs,
		before_all,
		options: options_to_merge,
	} = if let Some(before_all_script) = agent.before_all_script() {
		let lua_engine = runtime.new_lua_engine_with_ctx(&literals)?;
		let lua_scope = lua_engine.create_table()?;
		let lua_inputs = inputs.clone().map(Value::Array).unwrap_or_default();
		lua_scope.set("inputs", lua_engine.serde_to_lua_value(lua_inputs)?)?;
		lua_scope.set("options", agent.options_as_ref())?;

		let lua_value = lua_engine.eval(before_all_script, Some(lua_scope), Some(&[agent.file_dir()?.as_str()]))?;
		let before_all_res = serde_json::to_value(lua_value)?;

		match AipackCustom::from_value(before_all_res)? {
			// it is an skip action
			FromValue::AipackCustom(AipackCustom::Skip { reason }) => {
				let reason_msg = reason.map(|reason| format!(" (Reason: {reason})")).unwrap_or_default();
				hub.publish(HubEvent::info_short(format!(
					"Aipack Skip inputs at Before All section{reason_msg}"
				)))
				.await;
				return Ok(RunAgentResponse::default());
			}

			// it is before_all_response, so, we eventually override the inputs
			FromValue::AipackCustom(AipackCustom::BeforeAllResponse(BeforeAllResponse {
				inputs: inputs_ov,
				before_all,
				options,
			})) => BeforeAllResponse {
				inputs: inputs_ov.or(inputs),
				before_all,
				options,
			},

			// if it is another AipackCustom, we throw error
			FromValue::AipackCustom(other) => {
				return Err(Error::custom(format!(
					"Aipack custom '{}' not supported at the Before All stage",
					other.as_ref()
				)));
			}

			// just plane value
			FromValue::OriginalValue(value) => BeforeAllResponse {
				inputs,
				before_all: Some(value),
				options: None,
			},
		}
	} else {
		BeforeAllResponse {
			inputs,
			before_all: None,
			options: None,
		}
	};

	// Normalize the inputs, so, if empty, we have one input of value Value::Null
	let inputs = inputs.unwrap_or_else(|| vec![Value::Null]);
	// The default of
	let agent: Agent = match options_to_merge {
		Some(options_to_merge) => {
			let options_to_merge: AgentOptions = serde_json::from_value(options_to_merge)?;
			let options_ov = agent.options_as_ref().merge_new(options_to_merge)?;
			agent.new_merge(options_ov)?
		}
		None => agent,
	};
	let before_all = before_all.unwrap_or_default();

	// -- Print the run info
	let genai_info = get_genai_info(&agent);
	// display relative agent path if possible
	let agent_path = match get_display_path(agent.file_path(), runtime.dir_context()) {
		Ok(path) => path.to_string(),
		Err(_) => agent.file_path().to_string(),
	};

	// Show the message
	let model_str: &str = agent.model();
	let model_resolved_str: &str = agent.model_resolved();
	let model_info = if model_str != model_resolved_str {
		format!("{model_str} ({model_resolved_str})")
	} else {
		model_resolved_str.to_string()
	};
	let agent_name = agent.name();

	let mut agent_info: Option<String> = None;
	if let AgentRef::PackRef(pack_ref) = agent.agent_ref() {
		let kind_pretty = pack_ref.repo_kind.to_pretty_lower();
		let pack_ref = pack_ref.to_string();
		agent_info = Some(format!(" ({pack_ref} from {kind_pretty})"))
	}
	let agent_info = agent_info.as_deref().unwrap_or_default();

	hub.publish(format!(
		"\nRunning agent command: {}{agent_info}\n                 from: {}\n   with default model: {}{genai_info}",
		agent_name, agent_path, model_info
	))
	.await;

	// -- Initialize outputs for capture
	let mut captured_outputs: Option<Vec<(usize, Value)>> =
		if agent.after_all_script().is_some() || return_output_values {
			Some(Vec::new())
		} else {
			None
		};

	// -- Run the inputs
	let mut join_set = JoinSet::new();
	let mut in_progress = 0;

	let concurrency = agent.options().input_concurrency().unwrap_or(DEFAULT_CONCURRENCY);

	for (input_idx, input) in inputs.clone().into_iter().enumerate() {
		let runtime_clone = runtime.clone();
		let agent_clone = agent.clone();
		let before_all_clone = before_all.clone();
		let literals = literals.clone();

		let base_run_config_clone = run_base_options.clone();

		// Spawn tasks up to the concurrency limit
		join_set.spawn(async move {
			// Execute the command agent (this will perform do Data, Instruction, and Output stages)
			let run_input_response = run_command_agent_input(
				input_idx,
				&runtime_clone,
				&agent_clone,
				before_all_clone,
				input,
				&literals,
				&base_run_config_clone,
			)
			.await?;

			// Process the output
			let run_input_value = run_input_response.map(|v| v.into_value()).unwrap_or_default();
			let output = match AipackCustom::from_value(run_input_value)? {
				// if it is a skip, we skip
				FromValue::AipackCustom(AipackCustom::Skip { reason }) => {
					let reason_msg = reason.map(|reason| format!(" (Reason: {reason})")).unwrap_or_default();
					hub.publish(HubEvent::info_short(format!(
						"Aipack Skip input at Output stage{reason_msg}"
					)))
					.await;
					Value::Null
				}

				// Any other AipackCustom is not supported at output stage
				FromValue::AipackCustom(other) => {
					return Err(Error::custom(format!(
						"Aipack custom '{}' not supported at the Output stage",
						other.as_ref()
					)));
				}

				// Plain value passthrough
				FromValue::OriginalValue(value) => value,
			};

			Ok((input_idx, output))
		});

		in_progress += 1;

		// If we've reached the concurrency limit, wait for one task to complete
		if in_progress >= concurrency {
			if let Some(res) = join_set.join_next().await {
				in_progress -= 1;
				match res {
					Ok(Ok((input_idx, output))) => {
						if let Some(outputs_vec) = &mut captured_outputs {
							outputs_vec.push((input_idx, output));
						}
					}
					Ok(Err(e)) => return Err(e),
					Err(e) => return Err(Error::custom(format!("Error while running input. Cause {e}"))),
				}
			}
		}
	}

	// Wait for the remaining tasks to complete
	while in_progress > 0 {
		if let Some(res) = join_set.join_next().await {
			in_progress -= 1;
			match res {
				Ok(Ok((input_idx, output))) => {
					if let Some(outputs_vec) = &mut captured_outputs {
						outputs_vec.push((input_idx, output));
					}
				}
				Ok(Err(e)) => return Err(e),
				Err(e) => return Err(Error::custom(format!("Error while remaining input. Cause {e}"))),
			}
		}
	}

	// -- Post-process outputs
	let outputs = if let Some(mut captured_outputs) = captured_outputs {
		captured_outputs.sort_by_key(|(idx, _)| *idx);
		Some(captured_outputs.into_iter().map(|(_, v)| v).collect::<Vec<_>>())
	} else {
		None
	};

	// -- Run the after all
	let after_all = if let Some(after_all_script) = agent.after_all_script() {
		let outputs_value = if let Some(outputs) = outputs.as_ref() {
			Value::Array(outputs.clone())
		} else {
			Value::Null
		};

		let lua_engine = runtime.new_lua_engine_with_ctx(&literals)?;
		let lua_scope = lua_engine.create_table()?;
		let inputs = Value::Array(inputs);
		lua_scope.set("inputs", lua_engine.serde_to_lua_value(inputs)?)?;
		// Will be Value::Null if outputs were not collected
		lua_scope.set("outputs", lua_engine.serde_to_lua_value(outputs_value)?)?;
		lua_scope.set("before_all", lua_engine.serde_to_lua_value(before_all)?)?;
		lua_scope.set("options", agent.options_as_ref())?;

		let lua_value = lua_engine.eval(after_all_script, Some(lua_scope), Some(&[agent.file_dir()?.as_str()]))?;
		Some(serde_json::to_value(lua_value)?)
	} else {
		None
	};

	hub.publish(format!("\n======= COMPLETED: {}", agent.name())).await;

	Ok(RunAgentResponse { after_all, outputs })
}

/// Run the command agent input for the run_command_agent_inputs
/// Not public by design, should be only used in the context of run_command_agent_inputs
async fn run_command_agent_input(
	input_idx: usize,
	runtime: &Runtime,
	agent: &Agent,
	before_all: Value,
	input: impl Serialize,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<Option<RunAgentInputResponse>> {
	let hub = get_hub();

	// -- prepare the scope_input
	let input = serde_json::to_value(input)?;

	// get the eventual "._label" property of the input
	// try to get the path, name
	let label = get_input_label(&input).unwrap_or_else(|| format!("{input_idx}"));
	hub.publish(format!("\n==== Running input: {}", label)).await;

	let run_response = run_agent_input(runtime, agent, before_all, &label, input, literals, run_base_options).await?;

	// if the response value is a String, then, print it
	if let Some(response_txt) = run_response.as_ref().and_then(|r| r.as_str()) {
		// let short_text = truncate_with_ellipsis(response_txt, 72);
		hub.publish(format!("-> Agent Output:\n\n{response_txt}\n")).await;
	}

	hub.publish(format!("==== DONE (input: {})", label)).await;

	Ok(run_response)
}

// region:    --- RunCommandResponse

/// The response returned by a Run Command call.
/// TODO: Need to check why `outputs` is optional.
///       We might want to have an array of Null if no output or nil was returned (to keep in sync with inputs).
#[derive(Debug, Serialize, Default)]
pub struct RunAgentResponse {
	pub outputs: Option<Vec<Value>>,
	pub after_all: Option<Value>,
}

impl IntoLua for RunAgentResponse {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		let outputs = self.outputs.map(|v| serde_values_to_lua_values(lua, v)).transpose()?;
		let after_all = self.after_all.map(|v| serde_value_to_lua_value(lua, v)).transpose()?;
		table.set("outputs", outputs)?;
		table.set("after_all", after_all)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- RunCommandResponse

// region:    --- Support

/// Return the display path
/// - If .aipack/ or relative to workspace, then, relatively to workspace
/// - If ~/.aipack-base/ then, absolute path
fn get_display_path(file_path: &str, dir_context: &DirContext) -> Result<SPath> {
	let file_path = SPath::new(file_path);

	if file_path.as_str().contains(".aipack-base") {
		Ok(file_path)
	} else {
		let spath = match dir_context.wks_dir() {
			Some(wks_dir) => file_path.try_diff(wks_dir)?,
			None => file_path,
		};
		Ok(spath)
	}
}

// endregion: --- Support

/// Workaround to expose the run_command_agent_input only for test.
#[cfg(test)]
pub async fn run_command_agent_input_for_test(
	input_idx: usize,
	runtime: &Runtime,
	agent: &Agent,
	before_all: Value,
	input: impl Serialize,
	run_base_options: &RunBaseOptions,
) -> Result<Option<RunAgentInputResponse>> {
	let literals = Literals::from_runtime_and_agent_path(runtime, agent)?;

	run_command_agent_input(
		input_idx,
		runtime,
		agent,
		before_all,
		input,
		&literals,
		run_base_options,
	)
	.await
}

// region:    --- Support

fn get_input_label(input: &Value) -> Option<String> {
	const LABEL_KEYS: &[&str] = &["path", "name", "label", "_label"];
	for &key in LABEL_KEYS {
		if let Ok(value) = input.x_get::<String>(key) {
			return Some(value);
		}
	}
	None
}

/// For the run commands info (before each input run)
fn get_genai_info(agent: &Agent) -> String {
	let mut genai_infos: Vec<String> = vec![];

	if let Some(temp) = agent.options().temperature() {
		genai_infos.push(format!("temperature: {temp}"));
	}

	if let Some(top_p) = agent.options().top_p() {
		genai_infos.push(format!("top_p: {top_p}"));
	}

	if genai_infos.is_empty() {
		"".to_string()
	} else {
		format!(" ({})", genai_infos.join(", "))
	}
}
// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
#[path = "../_tests/tests_run_agent_llm.rs"]
mod tests_run_agent_llm;

#[cfg(test)]
#[path = "../_tests/tests_run_agent_script.rs"]
mod tests_run_agent_script;

// endregion: --- Tests
