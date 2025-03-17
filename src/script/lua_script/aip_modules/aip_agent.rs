use crate::agent::{AgentOptions, find_agent};
use crate::run::{RunBaseOptions, Runtime, RuntimeContext, run_command_agent};
use crate::script::{LuaValueExt, lua_value_to_serde_value};
use crate::{Error, Result};
use mlua::{FromLua, IntoLua, Lua, Table, Value};
use serde_json::Value as JsonValue;

pub fn init_module(lua: &Lua, runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let ctx = runtime_context.clone();
	let agent_run = lua.create_function(move |lua, (agent_name, options): (String, Option<Value>)| {
		aip_agent_run(lua, &ctx, agent_name, options)
	})?;

	table.set("run", agent_run)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Run another agent and get its response
///
/// ```lua
/// -- Run an agent with a single input
/// local response = aip.agent.run("agent-name", { inputs = "hello" })
///
/// -- Run an agent with multiple inputs
/// local response = aip.agent.run("agent-name", { inputs = {"input1", "input2"} })
///
/// -- Run an agent with structured inputs
/// local response = aip.agent.run("agent-name", {
///   inputs = {
///     { path = "file1.txt", content = "..." },
///     { path = "file2.txt", content = "..." }
///   }
/// })
/// ```
///
/// ### Returns
///
/// Returns the result of the agent execution, which will depend on the agent's output.
/// If the agent produces an AI response, it will return the AI response object.
/// If the agent has an output script, it will return the output from that script.
///
pub fn aip_agent_run(
	lua: &Lua,
	runtime_ctx: &RuntimeContext,
	agent_name: String,
	options: Option<Value>,
) -> mlua::Result<Value> {
	// -- parse the Lua Options to the the LuaAgentRunOptions with inputs and agent options
	let options = options
		.map(|opt| LuaAgentRunOptions::from_lua(opt, lua))
		.transpose()?
		.unwrap_or_default();
	// Normalize inputs to JsonValue format
	let inputs = options.inputs;
	let agent_options = options.agent_options;

	// -- Find agent and build run base options
	// NOTE: For now, do not pass through the caller baseOptions.
	// TODO: Might need to find a way to pass it through (perhaps via CTX, or a _aipack_.run_base_options)

	// Find and build the agent
	let agent = find_agent(&agent_name, runtime_ctx.dir_context())
		.map_err(|e| Error::custom(format!("Failed to find agent '{}': {}", agent_name, e)))?;

	// -- If we had a agent options, need to overrid the agent options.
	let agent = match agent_options {
		Some(agent_options) => agent.new_merge(agent_options)?,
		None => agent,
	};

	// -- Build the environment
	let run_base_options = RunBaseOptions::default();

	// Create a new "shadow runtime" with the same runtime_ctx
	let runtime = Runtime::from_runtime_context(runtime_ctx.clone());

	// Execute the tokio runtime blocking to run the command agent
	let result = tokio::task::block_in_place(|| {
		tokio::runtime::Handle::current()
			.block_on(async { run_command_agent(&runtime, agent, inputs, &run_base_options, true).await })
	})
	.map_err(|e| Error::custom(format!("Failed to run agent '{}': {}", agent_name, e)))?;

	// Process the result
	let run_command_response = result.into_lua(lua)?;

	Ok(run_command_response)
}

/// Options for the agent run function
#[derive(Debug, Default)]
pub struct LuaAgentRunOptions {
	/// Inputs to pass to the agent
	pub inputs: Option<Vec<JsonValue>>,

	/// TODO: need to implement agent options override
	pub agent_options: Option<AgentOptions>,
}

impl FromLua for LuaAgentRunOptions {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let inputs = value.x_get_value("inputs").map(lua_value_to_serde_value).transpose()?;

		let inputs = match inputs {
			Some(JsonValue::Array(values)) => Some(values),
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
		Ok(Self { inputs, agent_options })
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, run_reflective_agent};
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_agent_run_simple() -> Result<()> {
		// -- Setup & Fixtures
		let script = r#"
            local result = aip.agent.run("agent-script/agent-hello-world")
            return result
        "#;

		// -- Exec
		let mut res = run_reflective_agent(script, None).await?;

		// -- Check
		let mut outputs = res.x_remove::<serde_json::Value>("outputs")?;
		let len = outputs.as_array().map(|a| a.len()).ok_or("'outputs' should be array")?;
		assert_eq!(len, 1, "'outputs' should have one output");
		let output = outputs.x_remove::<String>("/0")?;
		assert_contains(&output, "Hello Wonderful World");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_agent_run_with_input() -> Result<()> {
		// -- Setup & Fixtures
		let script = r#"
            local result = aip.agent.run("agent-script/agent-hello", { inputs = {"John"} })
            return result
        "#;

		// -- Exec
		let mut res = run_reflective_agent(script, None).await?;

		// -- Check
		let mut outputs = res.x_remove::<serde_json::Value>("outputs")?;
		let len = outputs.as_array().map(|a| a.len()).ok_or("'ouputs' should be array")?;
		assert_eq!(len, 1, "'ouputs' should have one output");
		let output = outputs.x_remove::<String>("/0")?;
		assert_contains(&output, "Hello 'John' from agent-hello.aip");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_agent_run_with_options() -> Result<()> {
		// -- Setup & Fixtures
		let script = r#"
            local result = aip.agent.run("agent-script/agent-hello", 
               { inputs =  {"John"},
                 options = {model = "super-fast"}
               }
            )
            return result
        "#;

		// -- Exec
		let mut res = run_reflective_agent(script, None).await?;

		// -- Check
		let mut outputs = res.x_remove::<serde_json::Value>("outputs")?;
		let len = outputs.as_array().map(|a| a.len()).ok_or("'ouputs' should be array")?;
		assert_eq!(len, 1, "'ouputs' should have one output");
		let output = outputs.x_remove::<String>("/0")?;
		assert_contains(&output, "Hello 'John' from agent-hello.aip");
		assert_contains(&output, "options.model = super-fast");

		Ok(())
	}
}

// endregion: --- Tests
