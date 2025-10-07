//! Defines the `aip.tag` module, used in the Lua engine.
//!
//! This module provides functions for extracting custom tag blocks (e.g., `<FILE>...</FILE>`)
//! from text content.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.tag` module exposes functions to interact with and parse custom tagged content.
//!
//! ### Functions
//!
//! - `aip.tag.extract(content: string, tag_names: string | string[], options?: {extrude?: "content"}): list<TagBlock> | (list<TagBlock>, string)`

use crate::Result;
use crate::runtime::Runtime;
use crate::script::support::into_vec_of_strings;
use crate::support::text::TagContentIterator;
use crate::types::Extrude;
use crate::types::TagElem;
use mlua::{Error as LuaError, Lua, MultiValue, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let module = lua.create_table()?;
	module.set("extract", lua.create_function(tag_extract)?)?;

	Ok(module)
}

/// Extracts tagged blocks from a string.
///
/// If `options` contains `extrude = "content"`, returns `(list<TagBlock>, string)`.
/// Otherwise, returns `list<TagBlock>`.
fn tag_extract(lua: &Lua, (content, tag_names, options): (String, Value, Option<Table>)) -> mlua::Result<MultiValue> {
	let tag_names_vec = validate_and_normalize_tag_names(tag_names)?;
	let extrude = options.map_or(Ok(None), |options_v| Extrude::extract_from_table_value(&options_v))?;

	let (blocks, extruded) = extract_tag_blocks(&content, &tag_names_vec, extrude);

	let mut values = MultiValue::new();
	let blocks_table = lua.create_sequence_from(blocks)?;
	values.push_back(Value::Table(blocks_table));

	if let Some(extruded_content) = extruded {
		let extruded_lua_string = lua.create_string(&extruded_content)?;
		values.push_back(Value::String(extruded_lua_string));
	}

	Ok(values)
}

/// Validates and normalizes tag names derived from a Lua Value.
/// Ensures that tag names are provided (not nil), non-empty, and trims whitespace.
fn validate_and_normalize_tag_names(tag_names: Value) -> mlua::Result<Vec<String>> {
	if matches!(tag_names, Value::Nil) {
		return Err(LuaError::RuntimeError("aip.tag.extract requires tag_names".into()));
	}

	let names = into_vec_of_strings(tag_names, "aip.tag.extract tag_names")?;

	let mut trimmed_names = Vec::with_capacity(names.len());

	for name in names {
		let trimmed = name.trim();
		if trimmed.is_empty() {
			return Err(LuaError::RuntimeError(
				"tag_names cannot contain empty entries after trimming".into(),
			));
		}
		trimmed_names.push(trimmed.to_string());
	}

	if trimmed_names.is_empty() {
		return Err(LuaError::RuntimeError("tag_names list must not be empty".into()));
	}

	Ok(trimmed_names)
}

fn extract_tag_blocks(content: &str, tag_names: &[String], extrude: Option<Extrude>) -> (Vec<TagElem>, Option<String>) {
	let mut blocks: Vec<TagElem> = Vec::new();
	let mut extruded = extrude.map(|Extrude::Content| String::new());
	let mut last_idx: usize = 0;

	if !tag_names.is_empty() {
		let tag_refs: Vec<&str> = tag_names.iter().map(String::as_str).collect();

		for tag_content in TagContentIterator::new(content, &tag_refs) {
			if let Some(ref mut extruded_content) = extruded {
				if tag_content.start_idx > last_idx {
					extruded_content.push_str(&content[last_idx..tag_content.start_idx]);
				}
				last_idx = tag_content.end_idx + 1;
			}

			blocks.push(TagElem {
				tag: tag_content.tag_name.to_string(),
				attrs: None,
				content: tag_content.content.to_string(),
			});
		}
	}

	if let Some(ref mut extruded_content) = extruded
		&& last_idx < content.len()
	{
		extruded_content.push_str(&content[last_idx..]);
	}

	(blocks, extruded)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::init_module;
	use crate::_test_support::{eval_lua, setup_lua};

	#[tokio::test]
	async fn test_script_aip_tag_extract_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "tag").await?;
		let script = r#"
            local content = "Prefix <A>one</A> middle <B>two</B> suffix"
            local blocks, extruded = aip.tag.extract(content, {"A", "B"})
            return { blocks = blocks, extruded = extruded }
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let blocks = res.get("blocks").and_then(|v| v.as_array()).ok_or("Expected blocks array")?;
		assert_eq!(blocks.len(), 2);

		let block_a = blocks
			.first()
			.and_then(|v| v.as_object())
			.ok_or("Expected first block object")?;
		let block_b = blocks
			.get(1)
			.and_then(|v| v.as_object())
			.ok_or("Expected second block object")?;

		let block_a_name = block_a
			.get("tag")
			.and_then(|v| v.as_str())
			.ok_or("Expected name for first block")?;
		let block_a_content = block_a
			.get("content")
			.and_then(|v| v.as_str())
			.ok_or("Expected content for first block")?;
		let block_b_name = block_b
			.get("tag")
			.and_then(|v| v.as_str())
			.ok_or("Expected name for second block")?;
		let block_b_content = block_b
			.get("content")
			.and_then(|v| v.as_str())
			.ok_or("Expected content for second block")?;

		assert_eq!(block_a_name, "A");
		assert_eq!(block_a_content, "one");
		assert_eq!(block_b_name, "B");
		assert_eq!(block_b_content, "two");

		let extruded = res.get("extruded");
		// When not extruding, the second return value is nil, which translates to None/Null in serde
		assert!(extruded.is_none() || extruded.is_some_and(|v| v.is_null()));

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_tag_extract_with_extrude() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "tag").await?;
		let script = r#"
            local content = "Prefix <A>one</A> middle <B>two</B> suffix"
            local blocks, extruded = aip.tag.extract(content, {"A", "B"}, { extrude = "content" })
            return { blocks = blocks, extruded = extruded }
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let extruded = res.get("extruded").and_then(|v| v.as_str()).ok_or("Expected extruded string")?;
		assert_eq!(extruded, "Prefix  middle  suffix");

		let blocks = res.get("blocks").and_then(|v| v.as_array()).ok_or("Expected blocks array")?;
		assert_eq!(blocks.len(), 2);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_tag_extract_with_extrude_no_matches() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "tag").await?;
		let script = r#"
            local content = "No tags here."
            local blocks, extruded = aip.tag.extract(content, {"X"}, { extrude = "content" })
            return { blocks = blocks, extruded = extruded }
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let blocks_val = res.get("blocks").ok_or("Expected blocks array (key 'blocks' missing)")?;

		// When deserializing an empty sequence (list) from Lua back to Rust/JSON Value,
		// it often results in an empty object {} instead of an empty array [].
		// We check for expected emptiness across both array and object representations.
		let is_empty = blocks_val.as_array().map(|a| a.is_empty()).unwrap_or(false)
			|| blocks_val.as_object().map(|o| o.is_empty()).unwrap_or(false);

		if !is_empty {
			// If not empty, return a general error indicating blocks were found when none expected.
			return Err("Expected blocks array to be empty or represented as an empty object".into());
		}

		let extruded = res.get("extruded").and_then(|v| v.as_str()).ok_or("Expected extruded string")?;
		assert_eq!(extruded, "No tags here.");

		Ok(())
	}
}

// endregion: --- Tests
