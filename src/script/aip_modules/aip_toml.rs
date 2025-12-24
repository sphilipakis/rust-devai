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
//! - `aip.toml.stringify(content: table) -> string`
//!
//! ---
//!

use crate::runtime::Runtime;
use crate::script::{lua_value_to_serde_value, serde_value_to_lua_value};
use crate::support::tomls;
use crate::{Error, Result};
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let parse_fn = lua.create_function(move |lua, content: String| parse(lua, content))?;
	let stringify_fn = lua.create_function(move |lua, content: Value| stringify(lua, content))?;

	table.set("parse", parse_fn)?;
	table.set("stringify", stringify_fn)?;

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

/// ## Lua Documentation
/// ---
/// Stringify a table into a TOML string.
///
/// ```lua
/// -- API Signature
/// aip.toml.stringify(content: table): string
/// ```
///
/// Converts a Lua table into a TOML string, useful for generating configuration
/// files or persisting structured data.
///
/// ### Arguments
///
/// - `content: table` - The Lua table to stringify.
///
/// ### Returns
///
/// - `string` - A TOML-formatted string representing the provided table.
///
/// ### Example
///
/// ```lua
/// local obj = {
///     title = "Example",
///     owner = { name = "John" }
/// }
/// local toml_str = aip.toml.stringify(obj)
/// -- Result will include:
/// -- title = "Example"
/// -- [owner]
/// -- name = "John"
/// ```
///
/// ### Error
///
/// Returns an error if the table cannot be serialized into TOML.
///
/// ```ts
/// {
///   error: string  // Error message from TOML stringification, e.g., "aip.toml.stringify fail to stringify. ..."
/// }
/// ```
fn stringify(_lua: &Lua, content: Value) -> mlua::Result<String> {
	let json_value = lua_value_to_serde_value(content)?;
	tomls::stringify_json_value_to_toml_string(&json_value)
		.map_err(|err| Error::custom(format!("aip.toml.stringify fail to stringify. {err}")).into())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules;
	use crate::support::tomls;
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

	#[tokio::test]
	async fn test_script_lua_toml_stringify_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_toml::init_module, "toml").await?;
		let script = r#"
            local obj = {
                title = "Example",
                year = 2024
            }
            return aip.toml.stringify(obj)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let result = res.as_str().ok_or("Expected string result")?;
		assert_contains(result, r#"title = "Example""#);
		assert_contains(result, "year = 2024");
		let parsed = tomls::parse_toml_into_json(result)?;
		let expected = json!({
			"title": "Example",
			"year": 2024
		});
		assert_eq!(parsed, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_toml_stringify_nested_tables() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_toml::init_module, "toml").await?;
		let script = r#"
            local obj = {
                title = "Example",
                owner = {
                    name = "John",
                    email = "john@example.com"
                },
                tags = {"alpha", "beta"}
            }
            return aip.toml.stringify(obj)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let result = res.as_str().ok_or("Expected string result")?;
		assert_contains(result, "[owner]");
		assert_contains(result, r#"name = "John""#);
		assert_contains(result, r#"tags = ["alpha", "beta"]"#);
		let parsed = tomls::parse_toml_into_json(result)?;
		let expected = json!({
			"title": "Example",
			"owner": {
				"name": "John",
				"email": "john@example.com"
			},
			"tags": ["alpha", "beta"]
		});
		assert_eq!(parsed, expected);
		Ok(())
	}
}

// endregion: --- Tests
