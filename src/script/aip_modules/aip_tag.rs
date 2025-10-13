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
//! - `aip.tag.extract(content: string, tag_names: string | string[], options?: {extrude?: "content"}): list<TagElem> | (list<TagElem>, string)`

use crate::Result;
use crate::runtime::Runtime;
use crate::script::support::into_vec_of_strings;
use crate::support::tag::TagElemIter;
use crate::types::Extrude;
use crate::types::TagElem;
use mlua::{Error as LuaError, IntoLua, Lua, MultiValue, Table, Value};
use std::collections::HashMap;

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let module = lua.create_table()?;
	module.set("extract", lua.create_function(tag_extract)?)?;
	module.set("extract_as_map", lua.create_function(tag_extract_as_map)?)?;
	module.set("extract_as_multi_map", lua.create_function(tag_extract_as_multi_map)?)?;

	Ok(module)
}

/// Extracts tagged blocks from a string.
///
/// If `options` contains `extrude = "content"`, returns `(list<TagElem>, string)`.
/// Otherwise, returns `list<TagElem>`.
fn tag_extract(lua: &Lua, (content, tag_names, options): (String, Value, Option<Table>)) -> mlua::Result<MultiValue> {
	let tag_names_vec = validate_and_normalize_tag_names(tag_names)?;
	let extrude = options.map_or(Ok(None), |options_v| Extrude::extract_from_table_value(&options_v))?;

	let (blocks, extruded) = extract_tag_elems(&content, &tag_names_vec, extrude);

	let mut values = MultiValue::new();
	let blocks_table = lua.create_sequence_from(blocks)?;
	values.push_back(Value::Table(blocks_table));

	if let Some(extruded_content) = extruded {
		let extruded_lua_string = lua.create_string(&extruded_content)?;
		values.push_back(Value::String(extruded_lua_string));
	}

	Ok(values)
}

fn tag_extract_as_map(
	lua: &Lua,
	(content, tag_names, options): (String, Value, Option<Table>),
) -> mlua::Result<MultiValue> {
	let tag_names_vec = validate_and_normalize_tag_names(tag_names)?;
	let extrude = options.map_or(Ok(None), |options_v| Extrude::extract_from_table_value(&options_v))?;

	let (blocks, extruded) = extract_tag_elems(&content, &tag_names_vec, extrude);

	// Collect blocks into HashMap to ensure only the last block for a given tag name is kept.
	let blocks_map: HashMap<String, TagElem> = blocks.into_iter().map(|block| (block.tag.clone(), block)).collect();

	let map_table = lua.create_table()?;
	for (tag_name, block) in blocks_map {
		let block_value = block.into_lua(lua)?;
		map_table.set(tag_name, block_value)?;
	}

	let mut values = MultiValue::new();
	values.push_back(Value::Table(map_table));

	if let Some(extruded_content) = extruded {
		let extruded_lua_string = lua.create_string(&extruded_content)?;
		values.push_back(Value::String(extruded_lua_string));
	}

	Ok(values)
}

fn tag_extract_as_multi_map(
	lua: &Lua,
	(content, tag_names, options): (String, Value, Option<Table>),
) -> mlua::Result<MultiValue> {
	let tag_names_vec = validate_and_normalize_tag_names(tag_names)?;
	let extrude = options.map_or(Ok(None), |options_v| Extrude::extract_from_table_value(&options_v))?;

	let (blocks, extruded) = extract_tag_elems(&content, &tag_names_vec, extrude);

	let mut blocks_map: HashMap<String, Vec<TagElem>> = HashMap::new();
	for block in blocks {
		let tag_key = block.tag.clone();
		blocks_map.entry(tag_key).or_default().push(block);
	}

	let map_table = lua.create_table()?;
	for (tag_name, blocks_vec) in blocks_map {
		let blocks_sequence = lua.create_sequence_from(blocks_vec)?;
		map_table.set(tag_name, blocks_sequence)?;
	}

	let mut values = MultiValue::new();
	values.push_back(Value::Table(map_table));

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

fn extract_tag_elems(content: &str, tag_names: &[String], extrude: Option<Extrude>) -> (Vec<TagElem>, Option<String>) {
	let extrude_enabled = extrude.as_ref().is_some_and(|value| matches!(value, Extrude::Content));

	if tag_names.is_empty() {
		let extruded = if extrude_enabled {
			Some(content.to_string())
		} else {
			None
		};
		return (Vec::new(), extruded);
	}

	let tag_refs: Vec<&str> = tag_names.iter().map(String::as_str).collect();
	let iter = TagElemIter::with_tag_names(content, &tag_refs, extrude);
	let (elems, extruded_content) = iter.collect_elems_and_extruded_content();

	let extruded = if extrude_enabled { Some(extruded_content) } else { None };

	(elems, extruded)
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
	async fn test_script_aip_tag_extract_as_map_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "tag").await?;
		let script = r#"
            local content = "Prefix <A>one</A> middle <B>two</B> suffix <A>three</A>"
            local map = aip.tag.extract_as_map(content, {"A", "B"})
            return map
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let tag_a = res.get("A").and_then(|v| v.as_object()).ok_or("Expected map entry for tag A")?;
		let tag_b = res.get("B").and_then(|v| v.as_object()).ok_or("Expected map entry for tag B")?;

		let tag_a_content = tag_a
			.get("content")
			.and_then(|v| v.as_str())
			.ok_or("Expected content for tag A")?;
		let tag_b_content = tag_b
			.get("content")
			.and_then(|v| v.as_str())
			.ok_or("Expected content for tag B")?;

		assert_eq!(tag_a_content, "three");
		assert_eq!(tag_b_content, "two");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_tag_extract_as_multi_map_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "tag").await?;
		let script = r#"
            local content = "Prefix <A>one</A> middle <B>two</B> suffix <A>three</A>"
            local map = aip.tag.extract_as_multi_map(content, {"A", "B"})
            return map
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let tag_a = res.get("A").and_then(|v| v.as_array()).ok_or("Expected array for tag A")?;
		assert_eq!(tag_a.len(), 2);

		let tag_a_first = tag_a
			.first()
			.and_then(|v| v.as_object())
			.ok_or("Expected first block object for tag A")?;
		let tag_a_second = tag_a
			.get(1)
			.and_then(|v| v.as_object())
			.ok_or("Expected second block object for tag A")?;
		let tag_b = res.get("B").and_then(|v| v.as_array()).ok_or("Expected array for tag B")?;
		assert_eq!(tag_b.len(), 1);

		let tag_a_first_content = tag_a_first
			.get("content")
			.and_then(|v| v.as_str())
			.ok_or("Expected content for first tag A block")?;
		let tag_a_second_content = tag_a_second
			.get("content")
			.and_then(|v| v.as_str())
			.ok_or("Expected content for second tag A block")?;
		let tag_b_content = tag_b
			.first()
			.and_then(|v| v.as_object())
			.and_then(|v| v.get("content"))
			.and_then(|v| v.as_str())
			.ok_or("Expected content for tag B block")?;

		assert_eq!(tag_a_first_content, "one");
		assert_eq!(tag_a_second_content, "three");
		assert_eq!(tag_b_content, "two");

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
	async fn test_script_aip_tag_extract_as_map_with_extrude() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "tag").await?;
		let script = r#"
            local content = "Prefix <A>one</A> middle <B>two</B> suffix"
            local map, extruded = aip.tag.extract_as_map(content, {"A", "B"}, { extrude = "content" })
            return { map = map, extruded = extruded }
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let extruded = res.get("extruded").and_then(|v| v.as_str()).ok_or("Expected extruded string")?;
		assert_eq!(extruded, "Prefix  middle  suffix");

		let map = res.get("map").and_then(|v| v.as_object()).ok_or("Expected map object")?;
		let tag_a = map.get("A").and_then(|v| v.as_object()).ok_or("Expected map entry for tag A")?;
		let tag_b = map.get("B").and_then(|v| v.as_object()).ok_or("Expected map entry for tag B")?;

		let tag_a_content = tag_a
			.get("content")
			.and_then(|v| v.as_str())
			.ok_or("Expected content for tag A")?;
		let tag_b_content = tag_b
			.get("content")
			.and_then(|v| v.as_str())
			.ok_or("Expected content for tag B")?;

		assert_eq!(tag_a_content, "one");
		assert_eq!(tag_b_content, "two");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_tag_extract_as_multi_map_with_extrude() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "tag").await?;
		let script = r#"
            local content = "Prefix <A>one</A> middle <B>two</B> suffix <A>three</A>"
            local map, extruded = aip.tag.extract_as_multi_map(content, {"A", "B"}, { extrude = "content" })
            return { map = map, extruded = extruded }
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let extruded = res.get("extruded").and_then(|v| v.as_str()).ok_or("Expected extruded string")?;
		assert_eq!(extruded, "Prefix  middle  suffix ");

		let map = res.get("map").and_then(|v| v.as_object()).ok_or("Expected map object")?;
		let tag_a = map.get("A").and_then(|v| v.as_array()).ok_or("Expected array for tag A")?;
		let tag_b = map.get("B").and_then(|v| v.as_array()).ok_or("Expected array for tag B")?;
		assert_eq!(tag_a.len(), 2);
		assert_eq!(tag_b.len(), 1);

		let tag_a_first_content = tag_a
			.first()
			.and_then(|v| v.as_object())
			.and_then(|v| v.get("content"))
			.and_then(|v| v.as_str())
			.ok_or("Expected content for first tag A block")?;
		let tag_a_second_content = tag_a
			.get(1)
			.and_then(|v| v.as_object())
			.and_then(|v| v.get("content"))
			.and_then(|v| v.as_str())
			.ok_or("Expected content for second tag A block")?;
		let tag_b_content = tag_b
			.first()
			.and_then(|v| v.as_object())
			.and_then(|v| v.get("content"))
			.and_then(|v| v.as_str())
			.ok_or("Expected content for tag B block")?;

		assert_eq!(tag_a_first_content, "one");
		assert_eq!(tag_a_second_content, "three");
		assert_eq!(tag_b_content, "two");

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
