use crate::agent::find_agent;
use crate::run::{RunBaseOptions, RunSubAgentParams, run_agent};
use crate::{Error, Result};

pub async fn exec_run_agent(params: RunSubAgentParams) -> Result<()> {
	// Normalize inputs to JsonValue format
	let RunSubAgentParams {
		runtime,
		agent_dir,
		agent_name,
		inputs,
		agent_options,
		response_shot,
	} = params;

	// -- Find agent and build run base options
	// NOTE: For now, do not pass through the caller baseOptions.
	// TODO: Might need to find a way to pass it through (perhaps via CTX, or a _aipack_.run_base_options)

	// Find and build the agent
	let agent = find_agent(&agent_name, &runtime, agent_dir.as_ref())
		.map_err(|e| Error::custom(format!("Failed to find agent '{agent_name}': {e}")))?;

	// -- If we had a agent options, need to overrid the agent options.
	let agent = match agent_options {
		Some(agent_options) => agent.new_merge(agent_options)?,
		None => agent,
	};

	// -- Build the environment
	// NOTE: For now, do not inherit the parent run, But eventually mgith be past in the RunAgentParams
	let run_base_options = RunBaseOptions::default();

	let result = run_agent(&runtime, agent, inputs, &run_base_options, true)
		.await
		.map_err(|e| Error::custom(format!("Failed to run agent '{agent_name}': {e}")));

	match response_shot {
		Some(response_shot) => {
			match result {
				Ok(result) => {
					if let Err(err) = response_shot.send(Ok(result)).await {
						return Err(Error::custom(format!(
							"Failed to send response to agent '{agent_name}': {err}"
						)));
					}
				}
				Err(err) => {
					// Handle the error case
					if let Err(err) = response_shot.send(Err(Error::custom(err.to_string()))).await {
						return Err(Error::custom(format!(
							"Failed to send response to agent '{agent_name}': {err}"
						)));
					}
				}
			}
		}
		None => {
			result?;
		}
	}

	Ok(())
}
