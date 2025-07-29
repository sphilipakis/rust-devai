//! Defines the `run` and `task` pin helpers for Lua scripts.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This module adds two helper functions for recording pins at runtime.
//!
//! ### Functions
//!
//! - `aip.run.pin(iden?: string, priority?: number, content?: any): integer`  
//!   Creates a pin attached to the current **run** (requires `CTX.RUN_UID` to be set).
//!
//! - `aip.task.pin(iden?: string, priority?: number, content?: any): integer`  
//!   Creates a pin attached to the current **task** (requires both `CTX.RUN_UID` and `CTX.TASK_UID`).
//!
//! The functions return the numeric database identifier of the created pin.

use crate::Result;
use crate::runtime::Runtime;
use crate::store::rt_model::{PinBmc, PinForCreate, RuntimeCtx};
use mlua::{Lua, Table, Value};

/// Registers the `run.pin` and `task.pin` helpers in Lua.
pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	// -- run.pin
	{
		let rt = runtime.clone();
		let run_fn = lua.create_function(
			move |lua, (iden, priority, content): (Option<String>, Option<f64>, Option<Value>)| {
				create_pin(lua, &rt, /*for_task*/ false, iden, priority, content).map_err(mlua::Error::external)
			},
		)?;
		let run_tbl = lua.create_table()?;
		run_tbl.set("pin", run_fn)?;
		table.set("run", run_tbl)?;
	}

	// -- task.pin
	{
		let rt = runtime.clone();
		let task_fn = lua.create_function(
			move |lua, (iden, priority, content): (Option<String>, Option<f64>, Option<Value>)| {
				create_pin(lua, &rt, /*for_task*/ true, iden, priority, content).map_err(mlua::Error::external)
			},
		)?;
		let task_tbl = lua.create_table()?;
		task_tbl.set("pin", task_fn)?;
		table.set("task", task_tbl)?;
	}

	Ok(table)
}

// region:    --- Support

/// Shared implementation for both `run.pin` and `task.pin`.
fn create_pin(
	lua: &Lua,
	runtime: &Runtime,
	for_task: bool,
	iden: Option<String>,
	priority: Option<f64>,
	content: Option<Value>,
) -> Result<i64> {
	let ctx = RuntimeCtx::extract_from_global(lua)?;

	let mm = runtime.mm();
	let (run_id, task_id) = {
		let run_id = ctx.get_run_id(mm)?.ok_or("Cannot create pin – no RUN context available")?;
		let task_id = if for_task {
			Some(ctx.get_task_id(mm)?.ok_or("Cannot create pin – no TASK context available")?)
		} else {
			None
		};
		(run_id, task_id)
	};

	let content_str = match content {
		None | Some(Value::Nil) => None,
		Some(Value::String(s)) => Some(s.to_str()?.to_string()),
		Some(other) => {
			let json_val = serde_json::to_value(&other)
				.map_err(|e| crate::Error::custom(format!("Cannot serialise content: {e}")))?;
			Some(json_val.to_string())
		}
	};

	let pin_c = PinForCreate {
		run_id,
		task_id,
		iden,
		priority,
		content: content_str,
	};

	let id = PinBmc::create(mm, pin_c)?;
	Ok(id.as_i64())
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::run_reflective_agent_with_runtime;
	use crate::runtime::Runtime;
	use crate::store::rt_model::PinBmc;

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_run_pin_simple() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let fx_code = r#"
aip.run.pin("some-iden", 0, "Some pin content")		
return "OK"
			"#;

		// -- Exec
		let res = run_reflective_agent_with_runtime(fx_code, None, runtime.clone()).await?;

		// -- Check
		assert_eq!(res.as_str().unwrap_or_default(), "OK");
		// check pins
		let pins = PinBmc::list_for_run(runtime.mm(), 0.into())?;
		assert_eq!(pins.len(), 1);
		assert_eq!(pins[0].content.as_deref().unwrap_or_default(), "Some pin content");

		Ok(())
	}
}

// endregion: --- Tests
