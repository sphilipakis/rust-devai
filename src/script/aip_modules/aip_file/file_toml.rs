//! Defines the `load_toml` function for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` TOML functions
//!
//! ### Functions
//!
//! - `aip.file.load_toml(path: string): table | value`

use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::serde_value_to_lua_value;
use crate::support::tomls;
use mlua::{Lua, Value};
use simple_fs::read_to_string;

/// ## Lua Documentation
///
/// Load a file, parse its content as TOML, and return the corresponding Lua value.
///
/// ```lua
/// -- API Signature
/// aip.file.load_toml(path: string): table | value
/// ```
///
/// Loads the content of the file specified by `path`, parses it as TOML,
/// and converts the result into a Lua value (typically a table, but can contain
/// strings, numbers, booleans, or nested tables depending on the TOML content).
/// The path is resolved relative to the workspace root.
///
/// ### Arguments
///
/// - `path: string`: The path to the TOML file, relative to the workspace root.
///
/// ### Returns
///
/// - `table | value`: A Lua value representing the parsed TOML content.
///
/// ### Example
///
/// ```lua
/// -- Assuming 'Config.toml' contains:
/// -- title = "Example"
/// -- [owner]
/// -- name = "John"
///
/// local config = aip.file.load_toml("Config.toml")
/// print(config.title)        -- Output: Example
/// print(config.owner.name)   -- Output: John
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The file cannot be found or read.
/// - The file content is not valid TOML.
/// - The TOML value cannot be converted to a Lua value.
///
/// ```ts
/// {
///   error: string // Error message (e.g., file not found, TOML parse error)
/// }
/// ```
pub(super) fn file_load_toml(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	// Resolve the path relative to the workspace directory
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	let content = read_to_string(&full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.load_toml - Failed to read toml file '{path}'.\nCause: {e}",
		))
	})?;

	let toml_value = tomls::parse_toml_into_json(&content).map_err(|e| {
		Error::from(format!(
			"aip.file.load_toml - Failed to parse toml file '{path}'.\nCause: {e}",
		))
	})?;

	let lua_value = serde_value_to_lua_value(lua, toml_value)?;

	Ok(lua_value)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{
		assert_contains, clean_sanbox_01_tmp_file, create_sanbox_01_tmp_file, run_reflective_agent,
	};
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_file_load_toml_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file(
			"test_lua_file_load_toml_ok.toml",
			r#"
title = "Test TOML"
enabled = true
values = [1, 2, 3]

[owner]
name = "Owner Name"
tags = ["alpha", "beta"]

[[servers]]
name = "alpha"
port = 8080

[[servers]]
name = "beta"
port = 9090
"#,
		)?;
		let fx_path = fx_file.as_str();

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_toml("{fx_path}")"#), None).await?;

		// -- Check
		assert_eq!(res.x_get_str("title")?, "Test TOML");
		assert!(res.x_get_bool("enabled")?);
		let values = res
			.get("values")
			.ok_or("should have values")?
			.as_array()
			.ok_or("values should be array")?;
		assert_eq!(values.len(), 3);
		assert_eq!(values[0].as_i64().ok_or("should have i64")?, 1);

		let owner = res.get("owner").ok_or("should have owner")?;
		assert_eq!(owner.x_get_str("name")?, "Owner Name");
		let tags = owner
			.get("tags")
			.ok_or("owner should have tags")?
			.as_array()
			.ok_or("tags should be array")?;
		assert_eq!(tags[0].as_str().ok_or("tag should be string")?, "alpha");

		let servers = res
			.get("servers")
			.ok_or("should have servers")?
			.as_array()
			.ok_or("servers should be array")?;
		assert_eq!(servers.len(), 2);
		assert_eq!(servers[1].x_get_str("name")?, "beta");
		assert_eq!(servers[1].x_get_i64("port")?, 9090);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_toml_file_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/non_existent_file.toml";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_toml("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(&err.to_string(), "aip.file.load_toml - Failed to read toml file");
		assert_contains(&err.to_string(), "non_existent_file.toml");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_toml_invalid_toml() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file(
			"test_lua_file_load_toml_invalid.toml",
			r#"
title = "Test
"#,
		)?;
		let fx_path = fx_file.as_str();

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_toml("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(&err.to_string(), "aip.file.load_toml - Failed to parse toml file");
		assert_contains(&err.to_string(), fx_path);

		// -- Clean
		clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}
}

// endregion: --- Tests
