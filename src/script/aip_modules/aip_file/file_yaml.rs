//! Defines the `load_yaml` function for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` YAML functions
//!
//! ### Functions
//!
//! - `aip.file.load_yaml(path: string): list`

use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::support::yamls;
use mlua::{IntoLua, Lua, Value};
use simple_fs::read_to_string;

/// ## Lua Documentation
///
/// Load a file, parse its content as YAML, and return the corresponding Lua list of documents.
///
/// ```lua
/// -- API Signature
/// aip.file.load_yaml(path: string): list
/// ```
///
/// Loads the content of the file specified by `path`, parses it as YAML (supporting multiple documents
/// separated by `---`), and converts the result into a Lua list of tables.
/// The path is resolved relative to the workspace root.
///
/// ### Arguments
///
/// - `path: string`: The path to the YAML file, relative to the workspace root.
///
/// ### Returns
///
/// - `list: table`: A Lua list (table indexed from 1) where each element corresponds to a parsed YAML document.
///
/// ### Example
///
/// ```lua
/// -- Assuming 'data.yaml' contains:
/// -- name: Doc1
/// -- ---
/// -- name: Doc2
///
/// local docs = aip.file.load_yaml("data.yaml")
/// print(docs[1].name) -- Output: Doc1
/// print(docs[2].name) -- Output: Doc2
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The file cannot be found or read.
/// - The file content is not valid YAML.
/// - The YAML value cannot be converted to a Lua value.
pub(super) fn file_load_yaml(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	// Resolve the path relative to the workspace directory
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	let content = read_to_string(&full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.load_yaml - Failed to read yaml file '{path}'.\nCause: {e}",
		))
	})?;

	let yaml_docs = yamls::parse(&content).map_err(|e| {
		Error::from(format!(
			"aip.file.load_yaml - Failed to parse yaml file '{path}'.\nCause: {e}",
		))
	})?;

	let lua_value = yaml_docs.into_lua(lua)?;

	Ok(lua_value)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, create_sanbox_01_tmp_file, run_reflective_agent};
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_file_load_yaml_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file(
			"test_lua_file_load_yaml_ok.yaml",
			r#"
title: Test YAML
---
name: Doc2
"#,
		)?;
		let fx_path = fx_file.as_str();

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_yaml("{fx_path}")"#), None).await?;

		// -- Check
		let arr = res.as_array().ok_or("Should be array")?;
		assert_eq!(arr.len(), 2);
		assert_eq!(arr[0].x_get_str("title")?, "Test YAML");
		assert_eq!(arr[1].x_get_str("name")?, "Doc2");

		// -- Clean
		// comment out cleanup for inspection
		// clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_yaml_file_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/non_existent_file.yaml";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_yaml("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(&err.to_string(), "aip.file.load_yaml - Failed to read yaml file");
		assert_contains(&err.to_string(), "non_existent_file.yaml");

		Ok(())
	}
}

// endregion: --- Tests
