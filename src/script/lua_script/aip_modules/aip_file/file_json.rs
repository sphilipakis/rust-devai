//! Defines the `load_json` function for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation for `aip.file.load_json`
//!
//! The `aip.file.load_json` function loads a file's content, parses it as JSON, and returns the result as a Lua table/value.
//!
//! ### Function
//!
//! - `aip.file.load_json(path: string): table | value`

use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::lua_script::serde_value_to_lua_value;
use mlua::{Lua, Value};

/// ## Lua Documentation
///
/// Load a file, parse its content as JSON, and return the Lua value.
///
/// ```lua
/// -- API Signature
/// aip.file.load_json(path: string): table | value
/// ```
///
/// Loads the content of the file specified by `path`, parses it as JSON,
/// and converts the result into a Lua value (typically a table).
///
/// ### Arguments
///
/// - `path: string`: The path to the JSON file, relative to the workspace root.
///
/// ### Returns
///
/// Returns a Lua value (table, string, number, boolean, nil) representing the parsed JSON content.
///
/// ### Example
///
/// ```lua
/// -- Assuming 'config.json' contains {"port": 8080, "enabled": true}
/// local config = aip.file.load_json("config.json")
/// print(config.port)    -- Output: 8080
/// print(config.enabled) -- Output: true
/// ```
///
/// ### Error
///
/// Returns an error if the file cannot be found, read, or if the content is not valid JSON.
///
/// ```ts
/// {
///   error: string  // Error message (e.g., file not found, JSON parse error)
/// }
/// ```
pub(super) fn file_load_json(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	// Resolve the path relative to the workspace directory
	let full_path = runtime.dir_context().resolve_path(path.into(), PathResolver::WksDir)?;

	// Read the file content
	let content = std::fs::read_to_string(&full_path)
		.map_err(|e| Error::from(format!("Failed to read file '{}'. Cause: {}", full_path, e)))?;

	// Parse the JSON content
	let json_value: serde_json::Value = serde_json::from_str(&content)
		.map_err(|e| Error::from(format!("Failed to parse JSON from file '{}'. Cause: {}", full_path, e)))?;

	// Convert the serde_json::Value to mlua::Value
	let lua_value = serde_value_to_lua_value(lua, json_value)?;

	Ok(lua_value)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, run_reflective_agent};
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_file_load_json_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/test_load_json.json"; // Relative to tests-data/sandbox-01/

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_json("{fx_path}")"#), None).await?;

		// -- Check
		assert_eq!(res.x_get_str("name")?, "Test JSON");
		assert_eq!(res.x_get_f64("version")?, 1.2);
		assert!(res.x_get_bool("enabled")?, "enabled should be true");

		let items = res
			.get("items")
			.ok_or("should have items")?
			.as_array()
			.ok_or("should be array")?;
		assert_eq!(items.len(), 2);
		assert_eq!(items[0].as_str().ok_or("should have item")?, "item1");
		assert_eq!(items[1].as_str().ok_or("should have item")?, "item2");

		let nested: &serde_json::Value = res.get("nested").ok_or("should have nested")?;
		assert_eq!(nested.x_get_str("key")?, "value");

		// TODO: need to see how we want to test this one
		// let nullable = res.get("nullable");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_json_file_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/non_existent_file.json";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_json("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(&err.to_string(), "Failed to read file");
		assert_contains(&err.to_string(), "non_existent_file.json");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_json_invalid_json() -> Result<()> {
		// -- Setup & Fixtures
		// Use an existing text file that is not JSON
		let fx_path = "file-01.txt"; // Content is "hello world file 01"

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_json("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(&err.to_string(), "Failed to parse JSON");
		assert_contains(&err.to_string(), fx_path);

		Ok(())
	}
}

// endregion: --- Tests
