//! Defines the `json` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//! The `json` module exposes functions to parse and stringify JSON content.
//!
//! ### Functions
//!
//! - `aip.json.parse(content: string) -> table`
//! - `aip.json.parse_ndjson(content: string) -> table[]`
//! - `aip.json.stringify(content: table) -> string`
//! - `aip.json.stringify_pretty(content: table) -> string`
//! - `aip.json.stringify_to_line(content: table) -> string` (deprecated alias for `stringify`)

use crate::runtime::Runtime;
use crate::script::lua_value_to_serde_value;
use crate::script::serde_value_to_lua_value;
use crate::support::jsons::parse_ndjson_from_reader;
use crate::{Error, Result};
use mlua::{Lua, Table, Value};
use std::io::BufReader;

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let parse_fn = lua.create_function(move |lua, content: String| parse(lua, content))?;
	let parse_ndjson_fn = lua.create_function(move |lua, content: String| parse_ndjson(lua, content))?;
	let stringify_fn = lua.create_function(move |lua, content: Value| stringify(lua, content))?;
	let stringify_pretty_fn = lua.create_function(move |lua, content: Value| stringify_pretty(lua, content))?;
	// stringify_to_line is now an alias for stringify
	let stringify_to_line_fn = stringify_fn.clone();

	table.set("parse", parse_fn)?;
	table.set("parse_ndjson", parse_ndjson_fn)?;
	table.set("stringify", stringify_fn)?;
	table.set("stringify_pretty", stringify_pretty_fn)?;

	// deprecated, should use stringify
	table.set("stringify_to_line", stringify_to_line_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Parse a JSON string into a table.
///
/// ```lua
/// -- API Signature
/// aip.json.parse(content: string): table
/// ```
///
/// Parse a JSON string into a table that can be used in the Lua script.
///
/// ### Example
///
/// ```lua
/// local json_str = '{"name": "John", "age": 30}'
/// local obj = aip.json.parse(json_str)
/// print(obj.name) -- prints "John"
/// ```
///
/// ### Returns
///
/// Returns a table representing the parsed JSON.
///
/// ### Error
///
/// ```ts
/// {
///   error: string  // Error message from JSON parsing
/// }
/// ```
fn parse(lua: &Lua, content: String) -> mlua::Result<Value> {
	match serde_json::from_str::<serde_json::Value>(&content) {
		Ok(val) => serde_value_to_lua_value(lua, val).map_err(|e| e.into()),
		Err(err) => Err(Error::custom(format!("aip.json.parse failed. {}", err)).into()),
	}
}

/// ## Lua Documentation
///
/// Parse a newline-delimited JSON (NDJSON) string into an array of tables.
///
/// ```lua
/// -- API Signature
/// aip.json.parse_ndjson(content: string): table[]
/// ```
///
/// Parses a string containing multiple JSON objects separated by newlines.
/// Each line should be a valid JSON object. Empty lines are skipped.
///
/// ### Example
///
/// ```lua
/// local ndjson_str = '{"name": "John", "age": 30}\n{"name": "Jane", "age": 25}'
/// local list = aip.json.parse_ndjson(ndjson_str)
/// -- list will be:
/// -- {
/// --   { name = "John", age = 30 },
/// --   { name = "Jane", age = 25 }
/// -- }
/// print(list[1].name) -- prints "John"
/// print(list[2].name) -- prints "Jane"
/// ```
///
/// ### Returns
///
/// Returns a table (acting as an array) where each element is a table representing a parsed JSON object from a line.
///
/// ### Error
///
/// ```ts
/// {
///   error: string  // Error message if any line fails JSON parsing
/// }
/// ```
fn parse_ndjson(lua: &Lua, content: String) -> mlua::Result<Value> {
	let reader = BufReader::new(content.as_bytes());
	match parse_ndjson_from_reader(reader) {
		Ok(values) => {
			let lua_value = serde_value_to_lua_value(lua, values)?;
			Ok(lua_value)
		}
		Err(err) => Err(Error::custom(format!("aip.json.parse_ndjson failed. {}", err)).into()),
	}
}

/// ## Lua Documentation
///
/// Stringify a table into a single line JSON string.
///
/// Good for newline json or compact representation.
///
/// ```lua
/// -- API Signature
/// aip.json.stringify(content: table): string
/// ```
///
/// Convert a table into a single line JSON string.
///
/// ### Example
///
/// ```lua
/// local obj = {
///     name = "John",
///     age = 30
/// }
/// local json_str = aip.json.stringify(obj)
/// -- Result will be:
/// -- {"name":"John","age":30}
/// ```
///
/// ### Returns
///
/// Returns a single line JSON string.
///
/// ### Error
///
/// ```ts
/// {
///   error: string  // Error message from JSON stringification
/// }
/// ```
fn stringify(_lua: &Lua, content: Value) -> mlua::Result<String> {
	let json_value = lua_value_to_serde_value(content)?;
	match serde_json::to_string(&json_value) {
		Ok(str) => Ok(str),
		Err(err) => Err(Error::custom(format!("aip.json.stringify fail to stringify. {}", err)).into()),
	}
}

/// ## Lua Documentation
///
/// Stringify a table into a JSON string with pretty formatting.
///
/// ```lua
/// -- API Signature
/// aip.json.stringify_pretty(content: table): string
/// ```
///
/// Convert a table into a JSON string with pretty formatting using 2 spaces indentation.
///
/// ### Example
///
/// ```lua
/// local obj = {
///     name = "John",
///     age = 30
/// }
/// local json_str = aip.json.stringify_pretty(obj)
/// -- Result will be:
/// -- {
/// --   "name": "John",
/// --   "age": 30
/// -- }
/// ```
///
/// ### Returns
///
/// Returns a formatted JSON string.
///
/// ### Error
///
/// ```ts
/// {
///   error: string  // Error message from JSON stringification
/// }
/// ```
fn stringify_pretty(_lua: &Lua, content: Value) -> mlua::Result<String> {
	let json_value = lua_value_to_serde_value(content)?;
	match serde_json::to_string_pretty(&json_value) {
		Ok(str) => Ok(str),
		Err(err) => Err(Error::custom(format!("aip.json.stringify_pretty fail to stringify. {}", err)).into()),
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, assert_not_contains, eval_lua, setup_lua};
	use crate::script::lua_script::aip_modules;
	use serde_json::json;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_script_lua_json_parse_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_json::init_module, "json")?;
		let script = r#"
            local content = '{"name": "John", "age": 30}'
            return aip.json.parse(content)
        "#;
		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.x_get_str("name")?, "John");
		assert_eq!(res.x_get_i64("age")?, 30);
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_json_parse_invalid() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_json::init_module, "json")?;
		let script = r#"
            local ok, err = pcall(function()
                local content = "{invalid_json}"
                return aip.json.parse(content)
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
			panic!("Expected error, got {:?}", res);
		};

		// -- Check
		let err_str = err.to_string();

		assert_contains(&err_str, "json.parse failed");
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_json_parse_ndjson_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_json::init_module, "json")?;
		let script = r#"
            local content = '{"name": "John", "age": 30}\n{"name": "Jane", "age": 25}'
            return aip.json.parse_ndjson(content)
        "#;
		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!([
			{"name": "John", "age": 30},
			{"name": "Jane", "age": 25}
		]);
		assert_eq!(res, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_json_parse_ndjson_empty_lines() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_json::init_module, "json")?;
		let script = r#"
            local content = '{"id": 1}\n\n{"id": 2}\n   \n{"id": 3}'
            return aip.json.parse_ndjson(content)
        "#;
		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!([
			{"id": 1},
			{"id": 2},
			{"id": 3}
		]);
		assert_eq!(res, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_json_parse_ndjson_invalid_json() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_json::init_module, "json")?;
		let script = r#"
            local ok, err = pcall(function()
                local content = '{"id": 1}\n{invalid_json}\n{"id": 3}'
                return aip.json.parse_ndjson(content)
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
			panic!("Expected error, got {:?}", res);
		};
		let err_str = err.to_string();
		assert_contains(&err_str, "aip.json.parse_ndjson failed");
		assert_contains(&err_str, "At line 2");
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_json_stringify_pretty_basic() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_json::init_module, "json")?;
		let script = r#"
            local obj = {
                name = "John",
                age = 30
            }
            return aip.json.stringify_pretty(obj)
        "#;
		// -- Exec
		let res = eval_lua(&lua, script)?;
		// -- Check
		let result = res.as_str().ok_or("Expected string result")?;
		let parsed: serde_json::Value = serde_json::from_str(result)?;
		assert_eq!(parsed["name"], "John");
		assert_eq!(parsed["age"], 30);
		assert!(result.contains("\n"), "Expected pretty formatting with newlines");
		assert!(result.contains("  "), "Expected pretty formatting with indentation");
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_json_stringify_pretty_complex() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_json::init_module, "json")?;
		let script = r#"
            local obj = {
                name = "John",
                age = 30,
                address = {
                    street = "123 Main St",
                    city = "New York"
                },
                hobbies = {"reading", "gaming"}
            }
            return aip.json.stringify_pretty(obj)
        "#;
		// -- Exec
		let res = eval_lua(&lua, script)?;
		// -- Check
		let result = res.as_str().ok_or("Expected string result")?;
		let parsed: serde_json::Value = serde_json::from_str(result)?;
		assert_eq!(parsed["name"], "John");
		assert_eq!(parsed["age"], 30);
		assert_eq!(parsed["address"]["street"], "123 Main St");
		assert_eq!(parsed["hobbies"][0], "reading");
		assert!(result.contains("\n"), "Expected pretty formatting with newlines");
		assert!(result.contains("  "), "Expected pretty formatting with indentation");
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_json_stringify_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_json::init_module, "json")?;
		let script = r#"
            local obj = {
                name = "John",
                age = 30,
                address = {
                    street = "123 Main St",
                    city = "New York"
                },
                hobbies = {"reading", "gaming"}
            }
            return aip.json.stringify(obj)
        "#;
		// -- Exec
		let res = eval_lua(&lua, script)?;
		// -- Check
		let result = res.as_str().ok_or("Expected string result")?;
		assert_contains(result, r#""name":"John""#);
		assert_not_contains(result, "\n");
		assert_not_contains(result, "  ");
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_json_stringify_to_line_alias() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_json::init_module, "json")?;
		let script = r#"
            local obj = {
                name = "John",
                age = 30,
                address = {
                    street = "123 Main St",
                    city = "New York"
                },
                hobbies = {"reading", "gaming"}
            }
            return aip.json.stringify_to_line(obj)
        "#;
		// -- Exec
		let res = eval_lua(&lua, script)?;
		// -- Check
		let result = res.as_str().ok_or("Expected string result")?;
		assert_contains(result, r#""name":"John""#);
		assert_not_contains(result, "\n");
		assert_not_contains(result, "  ");
		Ok(())
	}
}

// endregion: --- Tests
