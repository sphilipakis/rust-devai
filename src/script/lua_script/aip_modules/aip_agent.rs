use crate::Result;
use crate::exec::RunAgentParams;
use crate::run::RunAgentResponse;
use crate::runtime::Runtime;
use crate::support::event::oneshot;
use mlua::{IntoLua, Lua, Table, Value};

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let rt = runtime.clone();
	let agent_run = lua.create_function(move |lua, (agent_name, options): (String, Option<Value>)| {
		aip_agent_run(lua, &rt, agent_name, options)
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
	runtime: &Runtime,
	agent_name: String,
	run_options: Option<Value>,
) -> mlua::Result<Value> {
	// -- parse the Lua Options to the the LuaAgentRunOptions with inputs and agent options
	//TODO: Needs to give a resposne_oneshot below
	let (tx, rx) = oneshot::<RunAgentResponse>();
	let run_agent_params = match run_options {
		Some(run_options) => {
			RunAgentParams::new_from_lua_params(runtime.clone(), agent_name, Some(tx), run_options, lua)?
		}
		None => RunAgentParams::new_no_inputs(runtime.clone(), agent_name, Some(tx)),
	};

	let exec_sender = runtime.executor_sender();
	// Important to spawn off the send to make sure we do not have deadlock since we are in a sync.
	tokio::task::block_in_place(|| {
		let rt = tokio::runtime::Handle::try_current();
		match rt {
			Ok(rt) => rt.block_on(async {
				exec_sender.send(run_agent_params.into()).await;
			}),

			// NOTE: Here per design, we do not return error or break, as it is just for logging
			Err(err) => println!("AIPACK INTERNAL ERROR - no current tokio handle - {err}"),
		}
	});

	let run_agent_response = rx.recv()?;

	// Process the result
	let run_command_response = run_agent_response.into_lua(lua)?;

	Ok(run_command_response)
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
