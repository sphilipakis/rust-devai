//! Defines the `toml` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.toml` module exposes functions to parse TOML content.
//!
//! ### Functions
//!
//! - `aip.toml.parse(content: string) -> table`
//!   Parses a TOML string into a Lua table.
//!
//! ---
//!

use crate::runtime::Runtime;
use crate::script::serde_value_to_lua_value;
use crate::support::tomls;
use crate::{Error, Result};
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let parse_fn = lua.create_function(move |lua, content: String| parse(lua, content))?;

	table.set("parse", parse_fn)?;

	Ok(table)
}

/// ## Lua Documentation
/// ---
/// Parse a TOML string into a table.
///
/// ```lua
/// -- API Signature
/// aip.toml.parse(content: string): table
/// ```
///
/// Parse a TOML string into a table that can be used in the Lua script.
///
/// ### Arguments
///
/// - `content: string` - The TOML string to parse.
///
/// ### Returns
///
/// - `table` - A Lua table representing the parsed TOML structure. TOML tables
///   become Lua tables, arrays become Lua lists (tables with integer keys), and
///   scalar values map to the corresponding Lua primitives.
///
/// ### Example
///
/// ```lua
/// local toml_str = [[
/// title = "Example"
///
/// [owner]
/// name = "John"
/// ]]
/// local obj = aip.toml.parse(toml_str)
/// print(obj.title) -- prints "Example"
/// print(obj.owner.name) -- prints "John"
/// ```
///
/// ### Error
///
/// Returns an error if the input string is not valid TOML.
///
/// ```ts
/// {
///   error: string  // Error message from TOML parsing, e.g., "aip.toml.parse failed. ..."
/// }
/// ```
fn parse(lua: &Lua, content: String) -> mlua::Result<Value> {
	let json_value = match tomls::parse_toml_into_json(&content) {
		Ok(val) => val,
		Err(err) => return Err(Error::custom(format!("aip.toml.parse failed. {err}")).into()),
	};

	let lua_value = serde_value_to_lua_value(lua, json_value)?;

	Ok(lua_value)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules;
	use serde_json::json;

	#[tokio::test]
	async fn test_script_lua_toml_parse_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_toml::init_module, "toml").await?;
		let script = r#"
            local content = [[
title = "Example"
year = 2024

[owner]
name = "John"
]]
            return aip.toml.parse(content)
        "#;
		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!({
			"title": "Example",
			"year": 2024,
			"owner": {
				"name": "John"
			}
		});
		assert_eq!(res, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_toml_parse_invalid() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_toml::init_module, "toml").await?;
		let script = r#"
            local ok, err = pcall(function()
                local content = "invalid = [1, 2,,]"
                return aip.toml.parse(content)
            end)
            if ok then
                return "should not reach here"
            else
                return err
            end
        "#;
		// -- Exec
		let res = eval_lua(&lua, script);

		// -- Check
		let Err(err) = res else {
			panic!("Expected error, got {res:?}");
		};
		let err_str = err.to_string();
		assert_contains(&err_str, "aip.toml.parse failed");
		Ok(())
	}
}

// endregion: --- Tests
