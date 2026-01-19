use crate::_test_support::load_reflective_agent;
use crate::Result;
use crate::agent::Agent;
use crate::run::{RunBaseOptions, run_command_agent_input_for_test};
use crate::runtime::Runtime;
use serde::Serialize;
use serde_json::Value;

pub async fn run_reflective_agent(data_lua_code: &str, input: Option<Value>) -> Result<Value> {
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	run_reflective_agent_with_runtime(data_lua_code, input, runtime).await
}

/// A reflective agent just return the value of the data Lua section.
/// It's useful for testing Lua module functions.
///
pub async fn run_reflective_agent_with_runtime(
	data_lua_code: &str,
	input: Option<Value>,
	runtime: Runtime,
) -> Result<Value> {
	// -- create the run and task for test
	// TODO: Probably need to do an insert if not exist
	// NOTE: in the run_command_agent_input_for_test, the run/task ids are 0
	let db = runtime.mm().db();
	db.exec(
		"INSERT INTO run (id, uid, ctime, mtime) values (?, ?, ?, ?)",
		(0, uuid::Uuid::now_v7(), 0, 0),
	)?;
	db.exec(
		"INSERT INTO task (id, uid, ctime, mtime, run_id) values (?, ?, ?, ?, ?)",
		(0, uuid::Uuid::now_v7(), 0, 0, 0),
	)?;

	// -- Load the agent
	let agent = load_reflective_agent(data_lua_code)?;
	let input = if let Some(input) = input { input } else { Value::Null };

	let res =
		run_command_agent_input_for_test(0, &runtime, &agent, Value::Null, input, &RunBaseOptions::default()).await?;
	let res = res.unwrap_or_default();
	Ok(res)
}

pub async fn run_test_agent(runtime: &Runtime, agent: &Agent) -> Result<Value> {
	let res = run_command_agent_input_for_test(0, runtime, agent, Value::Null, Value::Null, &RunBaseOptions::default())
		.await?;
	let res = res.unwrap_or_default();
	Ok(res)
}

pub async fn run_test_agent_with_input(runtime: &Runtime, agent: &Agent, input: impl Serialize) -> Result<Value> {
	let res =
		run_command_agent_input_for_test(0, runtime, agent, Value::Null, input, &RunBaseOptions::default()).await?;
	let res = res.unwrap_or_default();
	Ok(res)
}
