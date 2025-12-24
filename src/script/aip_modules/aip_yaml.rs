//! Defines the `yaml` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.yaml` module exposes functions to parse and stringify YAML content.
//!
//! - Parse function will return nil if content is nil.
//! - Parse supports multi-document YAML and returns a list of tables.
//! - stringify will assume single document
//! - stringify_multi_docs will error if content is not an list
//!
//! ### Functions
//!
//! - `aip.yaml.parse(content: string | nil) -> table[] | nil`
//! - `aip.yaml.stringify(content: any) -> string`
//! - `aip.yaml.stringify_multi_docs(content: table) -> string`
//!
//! ---
//!

use crate::runtime::Runtime;
use crate::script::lua_value_to_serde_value;
use crate::support::yamls;
use crate::{Error, Result};
use mlua::{IntoLua, Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let parse_fn = lua.create_function(move |lua, content: Option<String>| parse(lua, content))?;
	let stringify_fn = lua.create_function(move |lua, content: Value| stringify(lua, content))?;
	let stringify_multi_docs_fn = lua.create_function(move |lua, content: Value| stringify_multi_docs(lua, content))?;

	table.set("parse", parse_fn)?;
	table.set("stringify", stringify_fn)?;
	table.set("stringify_multi_docs", stringify_multi_docs_fn)?;

	Ok(table)
}

/// ## Lua Documentation
/// ---
/// Parse a YAML string into a list of tables.
///
/// ```lua
/// -- API Signature
/// aip.yaml.parse(content: string | nil): table[] | nil
/// ```
///
/// Parse a YAML string, which can contain multiple documents separated by `---`,
/// into a Lua list of tables.
///
/// ### Arguments
///
/// - `content: string | nil` - The YAML string to parse. If nil, returns nil.
///
/// ### Returns
///
/// - `table[] | nil` - A Lua list (table with integer keys) where each element
///   represents one YAML document from the input.
///
/// ### Example
///
/// ```lua
/// local yaml_str = "name: John\n---\nname: Jane"
/// local docs = aip.yaml.parse(yaml_str)
/// print(docs[1].name) -- prints "John"
/// print(docs[2].name) -- prints "Jane"
/// ```
///
/// ### Error
///
/// Returns an error if the input string is not valid YAML.
///
/// ```ts
/// {
///   error: string  // Error message from YAML parsing, e.g., "aip.yaml.parse failed. ..."
/// }
/// ```
fn parse(lua: &Lua, content: Option<String>) -> mlua::Result<Value> {
	let Some(content) = content else {
		return Ok(Value::Nil);
	};

	let yaml_docs = yamls::parse(&content).map_err(|err| Error::custom(format!("aip.yaml.parse failed. {err}")))?;

	// YamlDocs implements IntoLua (as a table of values)
	let lua_value = yaml_docs.into_lua(lua)?;

	Ok(lua_value)
}

/// ## Lua Documentation
/// ---
/// Stringify a value into a YAML string.
///
/// ```lua
/// -- API Signature
/// aip.yaml.stringify(content: any): string
/// ```
///
/// Convert a Lua value (usually a table) into a YAML string.
///
/// ### Arguments
///
/// - `content: any` - The Lua value to stringify.
///
/// ### Returns
///
/// - `string` - A string containing the YAML representation of the input.
///
/// ### Example
///
/// ```lua
/// local obj = { name = "John", age = 30 }
/// local yaml_str = aip.yaml.stringify(obj)
/// ```
///
/// ### Error
///
/// Returns an error if the value cannot be serialized into YAML.
fn stringify(_lua: &Lua, content: Value) -> mlua::Result<String> {
	let json_value = lua_value_to_serde_value(content)?;
	yamls::stringify(&json_value)
		.map_err(|err| Error::custom(format!("aip.yaml.stringify fail to stringify. {err}")).into())
}

/// ## Lua Documentation
/// ---
/// Stringify a list of tables into a multi-document YAML string.
///
/// ```lua
/// -- API Signature
/// aip.yaml.stringify_multi_docs(content: table): string
/// ```
///
/// Converts a Lua list of tables into a single YAML string where each table
/// becomes a separate YAML document separated by `---`.
///
/// ### Arguments
///
/// - `content: table` - A Lua list of tables to stringify.
///
/// ### Returns
///
/// - `string` - A multi-document YAML string.
///
/// ### Error
///
/// Returns an error if serialization fails or if the content is not a list.
fn stringify_multi_docs(_lua: &Lua, content: Value) -> mlua::Result<String> {
	let json_value = lua_value_to_serde_value(content)?;

	if let serde_json::Value::Array(values) = json_value {
		yamls::stringify_multi(&values)
			.map_err(|err| Error::custom(format!("aip.yaml.stringify_multi_docs fail to stringify. {err}")).into())
	} else {
		Err(Error::custom("aip.yaml.stringify_multi_docs failed. Content must be a list of tables.").into())
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules;

	#[tokio::test]
	async fn test_script_lua_yaml_parse_multi() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_yaml::init_module, "yaml").await?;
		let script = r#"
            local content = [[
name: Doc1
---
name: Doc2
]]
            return aip.yaml.parse(content)
        "#;
		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res[0]["name"], "Doc1");
		assert_eq!(res[1]["name"], "Doc2");
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_yaml_stringify_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_yaml::init_module, "yaml").await?;
		let script = r#"
            local obj = { name = "John", age = 30 }
            return aip.yaml.stringify(obj)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let s = res.as_str().unwrap();
		assert_contains(s, "name: John");
		assert_contains(s, "age: 30");
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_yaml_stringify_multi() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_modules::aip_yaml::init_module, "yaml").await?;
		let script = r#"
            local docs = { {name = "D1"}, {name = "D2"} }
            return aip.yaml.stringify_multi_docs(docs)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let s = res.as_str().unwrap();
		assert_contains(s, "name: D1");
		assert_contains(s, "---");
		assert_contains(s, "name: D2");
		Ok(())
	}
}

// endregion: --- Tests
