//! Defines the `aip_flow` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua Documentation
//!
//! The `aip.flow` module allows controlling the flow of AIPACK agent execution.
//! The flow functions are designed to be returned so that the Agent Executor can act appropriately.
//!
//! ### Functions
//!
//! - `aip.flow.before_all_response(data: {inputs:? any[], options?: AgentOptions}) -> table`
//! - `aip.flow.skip(reason?: string) -> table`

use crate::Result;
use crate::runtime::Runtime;
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let before_all_response_fn = lua.create_function(aipack_before_all_response)?;
	table.set("before_all_response", before_all_response_fn)?;

	let skip_fn = lua.create_function(aipack_skip)?;
	table.set("skip", skip_fn)?;

	Ok(table)
}

// region: --- Lua Functions

/// ## Lua Documentation
///
/// Returns a response that overrides inputs.
///
/// ```lua
/// -- API Signature
/// aip.flow.before_all_response(data: {inputs:? any[], options?: AgentOptions}) -> table
/// ```
///
/// ### Arguments
///
/// - `data: table` Table containing the inputs and options.
///   - `inputs: any[]` An array of inputs to override.
///   - `options: AgentOptions` Agent options.
///
/// ### Returns
///
/// Returns a Lua table with the structure:
///
/// ```ts
/// {
///   _aipack_: {
///     kind: "BeforeAllResponse",
///     data: any
///   }
/// }
/// ```
///
/// ### Error
///
/// This function does not directly return any errors. Errors might occur during the creation of lua table.
fn aipack_before_all_response(lua: &Lua, data: Value) -> mlua::Result<Value> {
	let inner = lua.create_table()?;
	inner.set("kind", "BeforeAllResponse")?;
	inner.set("data", data)?;
	let outer = lua.create_table()?;
	outer.set("_aipack_", inner)?;

	Ok(Value::Table(outer))
}

/// ## Lua Documentation
///
/// Returns a response indicating a skip action for the input cycle.
///
/// ```lua
/// -- API Signature
/// aip.flow.skip(reason?: string) -> table
/// ```
///
/// ### Arguments
///
/// - `reason: string (optional)`:  The reason for skipping the input cycle.
///
/// ### Returns
///
/// Returns a Lua table with the structure:
///
/// ```ts
/// {
///   _aipack_: {
///     kind: "Skip",
///     data: {
///       reason: string | nil
///     }
///   }
/// }
/// ```
///
/// ### Error
///
/// This function does not directly return any errors. Errors might occur during the creation of lua table.
fn aipack_skip(lua: &Lua, reason: Option<String>) -> mlua::Result<Value> {
	let data = lua.create_table()?;
	data.set("reason", reason)?;

	let inner = lua.create_table()?;
	inner.set("kind", "Skip")?;
	inner.set("data", data)?;

	let outer = lua.create_table()?;
	outer.set("_aipack_", inner)?;

	Ok(Value::Table(outer))
}

// endregion: --- Lua Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::lua_script::aip_flow;
	use serde_json::Value;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_script_lua_aip_flow_before_all_response_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_flow::init_module, "flow")?;
		let script = r#"
			return aip.flow.before_all_response(123)
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let kind = res.x_get_str("/_aipack_/kind")?;
		assert_eq!(kind, "BeforeAllResponse");

		let data = res.x_get_i64("/_aipack_/data")?;
		assert_eq!(data, 123);
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_aip_flow_skip_with_reason() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_flow::init_module, "flow")?;
		let script = r#"
			return aip.flow.skip("Not applicable")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let kind = res.x_get_str("/_aipack_/kind")?;
		assert_eq!(kind, "Skip");

		let reason = res.x_get_str("/_aipack_/data/reason")?;
		assert_eq!(reason, "Not applicable");
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_aip_flow_skip_without_reason() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_flow::init_module, "flow")?;
		let script = r#"
			return aip.flow.skip()
		"#;

		// -- Exec
		let mut res = eval_lua(&lua, script)?;

		// -- Check
		let kind = res.x_get_str("/_aipack_/kind")?;
		assert_eq!(kind, "Skip");

		// NOTE: For now, even if we ask Option<Value>, on /_aipack_/data/reason, we get an error. Should probably be fix in value-ext
		let data = res.x_remove::<Value>("/_aipack_/data")?;
		let reason = data.x_get::<String>("reason").ok();
		assert!(reason.is_none(), "reason should be none");
		Ok(())
	}
}

// endregion: --- Tests
