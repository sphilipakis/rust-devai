//! Defines the `task` helpers for Lua scripts.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This module adds helper functions for updating task properties at runtime.
//!
//! ### Functions
//!
//! - `aip.task.set_label(label: string)`  
//! - `aip.task.pin(iden: string, content: string | {label?: string, content: string})`  
//! - `aip.task.pin(iden: string, priority: number, content: string | {label?: string, content: string})`  
//!

use crate::Result;
use crate::model::{RuntimeCtx, TaskBmc, TaskForUpdate};
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::script::support::create_pin;
use mlua::{Lua, Table, Value, Variadic};

/// Registers the `task.set_label` and `task.pin` helpers in Lua.
pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	// -- task.set_label
	{
		let rt = runtime.clone();
		let set_label_fn = lua
			.create_function(move |lua, label: Value| set_task_label(lua, &rt, label).map_err(mlua::Error::external))?;
		table.set("set_label", set_label_fn)?;
	}

	// -- task.pin
	{
		let rt = runtime.clone();
		let task_pin_fn = lua.create_function(move |lua, args: Variadic<Value>| {
			create_pin(lua, &rt, /*for_task*/ true, args).map_err(mlua::Error::external)
		})?;
		table.set("pin", task_pin_fn)?;
	}

	Ok(table)
}

fn set_task_label(lua: &Lua, runtime: &Runtime, label: Value) -> Result<()> {
	let label = label
		.x_as_lua_str()
		.ok_or("aip.task.set_label(label) – expected <string> for parameter `label`.")?
		.to_string();

	let ctx = RuntimeCtx::extract_from_global(lua)?;
	let mm = runtime.mm();

	let task_id = ctx
		.get_task_id(mm)?
		.ok_or("Cannot call 'aip.task.set_label(...)' outside of a task context.")?;

	let task_u = TaskForUpdate {
		label: Some(label),
		..Default::default()
	};

	TaskBmc::update(mm, task_id, task_u)?;

	Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::run_reflective_agent_with_runtime;
	use crate::runtime::Runtime;

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_task_set_label_simple() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let fx_code = r#"
aip.task.set_label("My Custom Label")
return "OK"
		"#;

		// -- Exec
		let res = run_reflective_agent_with_runtime(fx_code, None, runtime.clone()).await?;

		// -- Check
		assert_eq!(res.as_str().unwrap_or_default(), "OK");
		// check task label was updated
		let tasks = crate::model::TaskBmc::list_for_run(runtime.mm(), 1.into())?;
		assert_eq!(tasks.len(), 1);
		assert_eq!(tasks[0].label, Some("My Custom Label".to_string()));

		Ok(())
	}
}

// endregion: --- Tests
