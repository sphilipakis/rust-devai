//! Defines the `run` helpers for Lua scripts.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This module adds helper functions for updating run properties at runtime.
//!
//! ### Functions
//!
//! - `aip.run.set_label(label: string)`  
//! - `aip.run.pin(iden: string, content: string | {label?: string, content: string})`  
//! - `aip.run.pin(iden: string, priority: number, content: string | {label?: string, content: string})`  
//!   

use crate::Result;
use crate::model::{RunBmc, RunForUpdate, RuntimeCtx};
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::script::support::create_pin;
use mlua::{Lua, Table, Value, Variadic};

/// Registers the `run.set_label` and `run.pin` helpers in Lua.
pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	// -- run.set_label
	{
		let rt = runtime.clone();
		let set_label_fn = lua
			.create_function(move |lua, label: Value| set_run_label(lua, &rt, label).map_err(mlua::Error::external))?;
		table.set("set_label", set_label_fn)?;
	}

	// -- run.pin
	{
		let rt = runtime.clone();
		let run_pin_fn = lua.create_function(move |lua, args: Variadic<Value>| {
			create_pin(lua, &rt, /*for_task*/ false, args).map_err(mlua::Error::external)
		})?;
		table.set("pin", run_pin_fn)?;
	}

	Ok(table)
}

fn set_run_label(lua: &Lua, runtime: &Runtime, label: Value) -> Result<()> {
	let label = label
		.x_as_lua_str()
		.ok_or("aip.run.set_label(label) – expected <string> for parameter `label`.")?
		.to_string();

	let ctx = RuntimeCtx::extract_from_global(lua)?;
	let mm = runtime.mm();

	let run_id = ctx
		.get_run_id(mm)?
		.ok_or("Cannot call 'aip.run.set_label(...)' outside of a run context.")?;

	let run_u = RunForUpdate {
		label: Some(label),
		..Default::default()
	};

	RunBmc::update(mm, run_id, run_u)?;

	Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::run_reflective_agent_with_runtime;
	use crate::model::{PinBmc, RunBmc};
	use crate::runtime::Runtime;

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_run_set_label_simple() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let fx_code = r#"
aip.run.set_label("My Run Label")
return "OK"
		"#;

		// -- Exec
		let res = run_reflective_agent_with_runtime(fx_code, None, runtime.clone()).await?;

		// -- Check
		assert_eq!(res.as_str().unwrap_or_default(), "OK");
		// check run label was updated
		let runs = RunBmc::list(runtime.mm(), None)?;
		assert_eq!(runs.len(), 1);
		assert_eq!(runs[0].label, Some("My Run Label".to_string()));

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_run_pin_simple() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let fx_code = r#"
aip.run.pin("some-iden", "Some pin content")		
return "OK"
			"#;

		// -- Exec
		let res = run_reflective_agent_with_runtime(fx_code, None, runtime.clone()).await?;

		// -- Check
		assert_eq!(res.as_str().unwrap_or_default(), "OK");
		// check pins
		let pins = PinBmc::list_for_run(runtime.mm(), 0.into())?;
		assert_eq!(pins.len(), 1);
		assert_eq!(pins[0].content.as_deref().unwrap_or_default(), "\"Some pin content\"");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_run_pin_with_priority() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let fx_code = r#"
aip.run.pin("some-iden", 0.7, "Other content")
return "OK"
		"#;

		// -- Exec
		let res = run_reflective_agent_with_runtime(fx_code, None, runtime.clone()).await?;

		// -- Check
		assert_eq!(res.as_str().unwrap_or_default(), "OK");
		let pins = PinBmc::list_for_run(runtime.mm(), 0.into())?;
		assert_eq!(pins.len(), 1);
		assert_eq!(pins[0].priority, Some(0.7));

		Ok(())
	}
}

// endregion: --- Tests
