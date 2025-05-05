//! Defines the `load_json` and `load_ndjson` functions for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation for `aip.file.load_json`
//!
//! The `aip.file.load_json` function loads a file's content, parses it as JSON, and returns the result as a Lua table/value.
//!
//! ### Functions
//!
//! - `aip.file.load_json(path: string): table | value`
//! - `aip.file.load_ndjson(path: string): table`

use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::lua_script::serde_value_to_lua_value;
use crate::support::jsons::parse_ndjson_from_reader;
use mlua::{Lua, Value};
use std::fs::File;
use std::io::BufReader;

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
	let full_path = runtime.dir_context().resolve_path(path.clone().into(), PathResolver::WksDir)?;

	// Read the file content
	let content = std::fs::read_to_string(&full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.load_json - Failed to read file '{}'. Cause: {}",
			path, e
		))
	})?;

	// Parse the JSON content
	let json_value: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
		Error::from(format!(
			"aip.file.load_json - Failed to parse JSON from file '{}'. Cause: {}",
			path, e
		))
	})?;

	// Convert the serde_json::Value to mlua::Value
	let lua_value = serde_value_to_lua_value(lua, json_value)?;

	Ok(lua_value)
}

/// ## Lua Documentation
///
/// Load a file containing newline-delimited JSON (NDJSON), parse each line, and return a Lua array (table) of the results.
///
/// ```lua
/// -- API Signature
/// aip.file.load_ndjson(path: string): list
/// ```
///
/// Loads the content of the file specified by `path`, assuming each line is a valid JSON object.
/// Parses each line and returns a Lua array containing the parsed Lua values. Empty lines are skipped.
///
/// ### Arguments
///
/// - `path: string`: The path to the NDJSON file, relative to the workspace root.
///
/// ### Returns
///
/// Returns a Lua array (table) where each element corresponds to a successfully parsed JSON line from the file.
///
/// ### Example
///
/// ```lua
/// -- Assuming 'logs.ndjson' contains:
/// -- {"level": "info", "message": "Service started"}
/// -- {"level": "warn", "message": "Disk space low"}
///
/// local logs = aip.file.load_ndjson("logs.ndjson")
/// print(#logs) -- Output: 2
/// print(logs[1].message) -- Output: Service started
/// print(logs[2].level)   -- Output: warn
/// ```
///
/// ### Error
///
/// Returns an error if the file cannot be found or read, or if any non-empty line contains invalid JSON.
///
/// ```ts
/// {
///   error: string  // Error message (e.g., file not found, JSON parse error on line N)
/// }
/// ```
pub(super) fn file_load_ndjson(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	// Resolve the path relative to the workspace directory
	let full_path = runtime.dir_context().resolve_path(path.clone().into(), PathResolver::WksDir)?;

	// Open the file
	let file = File::open(&full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.load_ndjson - Failed to open file '{}'. Cause: {}",
			path, e
		))
	})?;
	let reader = BufReader::new(file);

	let json_value = parse_ndjson_from_reader(reader)
		.map_err(|err| Error::custom(format!("aip.file.load_ndjson - Failed.\nCause: {err}")))?;

	let lua_values = serde_value_to_lua_value(lua, json_value)?;

	Ok(lua_values)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{
		assert_contains, clean_sanbox_01_tmp_file, create_sanbox_01_tmp_file, run_reflective_agent,
	};
	use value_ext::JsonValueExt as _;

	// region:    --- load_json Tests

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

		// Check null value handling
		let nullable = res.get("nullable").ok_or("should have nullable")?;
		assert!(nullable.is_null(), "nullable should be json null");

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
		assert_contains(&err.to_string(), "aip.file.load_json - Failed to read file");
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
		assert_contains(&err.to_string(), "aip.file.load_json - Failed to parse JSON");
		assert_contains(&err.to_string(), fx_path);

		Ok(())
	}

	// endregion: --- load_json Tests

	// region:    --- load_ndjson Tests

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/test_load_ndjson.ndjson"; // Relative to tests-data/sandbox-01/

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await?;

		// -- Check
		let arr = res.as_array().ok_or("Result should be an array")?;
		assert_eq!(arr.len(), 3, "Should have 3 items from the ndjson file");

		// Check first item
		let item1 = arr.first().ok_or("Should have item 1")?;
		assert_eq!(item1.x_get_str("name")?, "item1");
		assert_eq!(item1.x_get_i64("value")?, 10);

		// Check second item
		let item2 = arr.get(1).ok_or("Should have item 2")?;
		assert_eq!(item2.x_get_str("name")?, "item2");
		assert_eq!(item2.x_get_i64("value")?, 20);
		assert!(item2.x_get_bool("active")?);

		// Check third item (with null and array)
		let item3 = arr.get(2).ok_or("Should have item 3")?;
		assert_eq!(item3.x_get_str("name")?, "item3");
		assert!(item3.get("value").ok_or("item3 should have value")?.is_null());
		let tags = item3
			.get("tags")
			.ok_or("item3 should have tags")?
			.as_array()
			.ok_or("tags should be array")?;
		assert_eq!(tags.len(), 2);
		assert_eq!(tags[0].as_str().ok_or("tag should be string")?, "a");
		assert_eq!(tags[1].as_str().ok_or("tag should be string")?, "b");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_file_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/non_existent_file.ndjson";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(&err.to_string(), "aip.file.load_ndjson - Failed to open file");
		assert_contains(&err.to_string(), "non_existent_file.ndjson");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_invalid_json_line() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file(
			"test_lua_file_load_jsonnd_invalid_json_line.ndjson",
			r#"{"valid": true}
invalid json line here
{"another_valid": 123}
"#,
		)?;
		let fx_path = fx_file.as_str();

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(
			&err.to_string(),
			"aip.file.load_ndjson - Failed to parse JSON on line 2",
		);
		assert_contains(&err.to_string(), fx_path);

		// -- Clean
		clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_empty_file() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file("test_lua_file_load_jsonnd_empty_file.ndjson", "")?;
		let fx_path = fx_file.as_str();

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await?;

		// -- Check
		let arr = res.as_array().ok_or("Result should be an array")?;
		assert_eq!(arr.len(), 0, "Should have 0 items from an empty file");

		// -- Clean
		clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_empty_lines_file() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file(
			"test_lua_file_load_jsonnd_empty_lines_file.ndjson",
			r#"

{"valid": true}


{"another": "valid"}

"#,
		)?;
		let fx_path = fx_file.as_str();

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await?;

		// -- Check
		let arr = res.as_array().ok_or("Result should be an array")?;
		assert_eq!(arr.len(), 2, "Should have 2 items, skipping empty lines");
		assert!(arr[0].x_get_bool("valid")?);
		assert_eq!(arr[1].x_get_str("another")?, "valid");

		// -- Clean
		clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}

	// endregion: --- load_ndjson Tests
}

// endregion: --- Tests
