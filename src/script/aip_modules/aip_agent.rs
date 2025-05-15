//! Defines the `aip_agent` module, used in the lua engine.
//!
//! This module provides functionalities to execute other agents from within the current Lua script,
//! allowing for complex workflows and modular agent design.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.agent` module exposes functions to run other agents from within a Lua script.
//!
//! ### Functions
//!
//! - `aip.agent.run(agent_name: string, options?: table): any`
//! - `aip.agent.extract_options(value: any): table | nil`

use crate::Result;
use crate::exec::RunAgentParams;
use crate::run::RunAgentResponse;
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::support::event::oneshot;
use mlua::{IntoLua, Lua, Table, Value};
use simple_fs::SPath;

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let rt = runtime.clone();
	let agent_run = lua.create_function(move |lua, (agent_name, options): (String, Option<Value>)| {
		aip_agent_run(lua, &rt, agent_name, options)
	})?;

	let extract_options = lua.create_function(move |lua, value: Value| aip_agent_extract_options(lua, value))?;

	table.set("run", agent_run)?;
	table.set("extract_options", extract_options)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Runs another agent and returns its response.
///
/// ```lua
/// -- API Signature
/// aip.agent.run(agent_name: string, options?: table): any
/// ```
///
/// Executes the agent specified by `agent_name`. The function waits for the called agent
/// to complete and returns its result. This allows for chaining agents together.
///
/// ### Arguments
///
/// - `agent_name: string`: The name of the agent to run. This can be a relative path
///   (e.g., `"../my-other-agent.aip"`) or a fully qualified pack reference
///   (e.g., `"my-ns@my-pack/feature/my-agent.aip"`). Relative paths are resolved
///   from the directory of the calling agent.
/// - `options?: table`: An optional table containing input data and agent options.
///   - `inputs?: string | list | table`: Input data for the agent. Can be a single string, a list of strings, or a table of structured inputs.
///   - `options?: table`: Agent-specific options. These options are passed directly to the called agent's
///     execution environment and can override settings defined in the called agent's `.aip` file.
///
/// #### Input Examples:
///
/// ```lua
/// -- Run an agent with a single string input
/// local response = aip.agent.run("agent-name", { inputs = "hello" })
///
/// -- Run an agent with multiple string inputs
/// local response = aip.agent.run("agent-name", { inputs = {"input1", "input2"} })
///
/// -- Run an agent with structured inputs (e.g., file records)
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
/// The result of the agent execution. The type of the returned value depends on the agent's output:
///
/// - If the agent produces an AI response without a specific output script, it returns a table representing the `AiResponse` object.
/// - If the agent has an output script, it returns the value returned by that output script.
///
/// ```ts
/// // Example structure of a returned AiResponse object (if no output script)
/// {
///   action: string, // e.g., "PrintToConsole", "WriteFiles"
///   outputs: any,   // Depends on the action/output
///   options: table  // Options used during the run
///   // ... other properties from AiResponse
/// }
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The `agent_name` is invalid or the agent file cannot be located/loaded.
/// - The options table contains invalid parameters.
/// - The execution of the called agent fails.
/// - An internal communication error occurs while waiting for the agent's result.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub fn aip_agent_run(
	lua: &Lua,
	runtime: &Runtime,
	agent_name: String,
	run_options: Option<Value>,
) -> mlua::Result<Value> {
	let agent_dir = get_agent_dir_from_lua(lua);

	// -- Parse the Lua Options to the the LuaAgentRunOptions with inputs and agent options
	let (tx, rx) = oneshot::<Result<RunAgentResponse>>();
	let run_agent_params = match run_options {
		Some(run_options) => {
			RunAgentParams::new_from_lua_params(runtime.clone(), agent_dir, agent_name, Some(tx), run_options, lua)?
		}
		None => RunAgentParams::new_no_inputs(runtime.clone(), agent_dir, agent_name, Some(tx)),
	};

	let exec_sender = runtime.executor_sender();
	// Important to spawn off the send to make sure we do not have deadlock since we are in a sync context.
	tokio::task::block_in_place(|| {
		let rt = tokio::runtime::Handle::try_current();
		match rt {
			Ok(rt) => rt.block_on(async {
				// Non-blocking send to the executor task
				let _ = exec_sender.send(run_agent_params.into()).await;
			}),

			// NOTE: Here per design, we do not return error or break, as it is just for logging
			Err(err) => println!("AIPACK INTERNAL ERROR - no current tokio handle - {err}"),
		}
	});

	// -- Wait for the result
	let run_agent_response = rx.recv()?;

	// -- Process the result
	let run_agent_response = run_agent_response?;
	let run_command_response = run_agent_response.into_lua(lua)?;

	Ok(run_command_response)
}

/// ## Lua Documentation
///
/// Extracts relevant agent options from a given Lua value.
///
/// ```lua
/// -- API Signature
/// aip.agent.extract_options(value: any): table | nil
/// ```
///
/// If the input `value` is a Lua table, this function creates a new table and copies
/// the following properties if they exist in the input table:
///
/// - `model`
/// - `model_aliases`
/// - `input_comcurrency`
/// - `temperature`
///
/// Other properties are ignored. If the input `value` is `nil` or not a table,
/// the function returns `nil`.
///
/// ### Arguments
///
/// - `value: any`: The Lua value from which to extract options.
///
/// ### Returns
///
/// A new Lua table containing the extracted options, or `nil` if the input
/// was `nil` or not a table.
pub fn aip_agent_extract_options(lua: &Lua, value: Value) -> mlua::Result<Value> {
	match value {
		Value::Table(table) => {
			let result_table = lua.create_table()?;

			// List of keys to copy
			let keys_to_copy = ["model", "model_aliases", "input_comcurrency", "temperature"];

			for key in keys_to_copy.iter() {
				if let Some(val) = table.x_get_value(key) {
					if val != Value::Nil {
						result_table.set(*key, val)?;
					}
				}
			}

			Ok(Value::Table(result_table))
		}
		_ => Ok(Value::Nil),
	}
}

// region:    --- Support

/// Helper function to get the calling agent's directory from the Lua CTX global.
fn get_agent_dir_from_lua(lua: &Lua) -> Option<SPath> {
	lua.globals()
		.x_get_value("CTX")?
		.x_get_string("AGENT_FILE_DIR")
		.map(|s| s.into())
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, run_reflective_agent, run_test_agent, setup_lua};
	use crate::agent::Agent;
	use crate::runtime::Runtime;
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_script_aip_agent_run_simple() -> Result<()> {
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
	async fn test_script_aip_agent_run_relative() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let agent_file = runtime
			.dir_context()
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join("agent-script/agent-calling-hello.aip");
		let agent = Agent::mock_from_file(agent_file.as_str())?;

		// -- Exec
		let mut res = run_test_agent(&runtime, &agent).await?;

		// -- Check
		let res = res.x_remove::<String>("res")?;
		assert_contains(&res, "Hello 'me' from agent-hello.aip");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_script_aip_agent_run_with_input() -> Result<()> {
		// -- Setup & Fixtures
		// Note: the agent is relative to sandbox-01/ because the run_reflective_agent mock agent is at the base
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
	async fn test_script_aip_agent_run_with_options() -> Result<()> {
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

	#[tokio::test]
	async fn test_script_aip_agent_extract_options_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "agent")?;
		let script = r#"
local big = {
		model = "gpt-4",
		temperature = 0.8,
		input_comcurrency = 5,
		some_other_prop = "ignore_me",
		model_aliases = { fast = "gpt-3.5" },
		nil_prop = nil -- should not be included
}
return aip.agent.extract_options(big)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.x_get_str("model")?, "gpt-4");
		assert_eq!(res.x_get_f64("temperature")?, 0.8);
		assert_eq!(res.x_get_i64("input_comcurrency")?, 5);
		assert!(
			res.x_get::<serde_json::Value>("some_other_prop").is_err(),
			"Should not have 'some_other_prop'"
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_agent_extract_options_string() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "agent")?;
		let script = r#"
return aip.agent.extract_options("Some string")
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert!(
			matches!(res, serde_json::Value::Null),
			"Should be nil but was: {:?}",
			res
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_agent_extract_options_nil() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "agent")?;
		let script = r#"
return aip.agent.extract_options(nil)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert!(
			matches!(res, serde_json::Value::Null),
			"Should be nil but was: {:?}",
			res
		);

		Ok(())
	}
}

// endregion: --- Tests
