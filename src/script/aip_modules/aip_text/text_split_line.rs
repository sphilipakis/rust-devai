//! Defines text splitting functions by lines for the `aip.text` Lua module.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This section of the `aip.text` module exposes functions for splitting text based on matching whole lines.
//!
//! ### Functions
//!
//! - `aip.text.split_first_line(content: string | nil, sep: string): (string | nil, string | nil)`
//! - `aip.text.split_last_line(content: string | nil, sep: string): (string | nil, string | nil)`

use crate::script::support::into_option_string;
use mlua::{Lua, MultiValue, String as LuaString, Value};

/// ## Lua Documentation
///
/// Splits a string into two parts based on the first line that exactly matches the separator.
/// The separator line itself is not included in either part.
/// If `content` is `nil`, returns `(nil, nil)`.
/// If no line matches the separator, returns `(original_content, nil)`.
///
/// ```lua
/// -- API Signature
/// aip.text.split_first_line(content: string | nil, sep: string): (string | nil, string | nil)
/// ```
///
/// ### Arguments
///
/// - `content: string | nil`: The string to split.
/// - `sep: string`: The exact string the line must match to be considered a separator.
///
/// ### Returns
///
/// A tuple containing the part before the separator line and the part after the separator line.
/// Empty parts are returned as empty strings.
/// Returns `(nil, nil)` if input `content` is `nil`.
/// Returns `(original_content, nil)` if no separator line is found.
///
/// ```ts
/// [string | nil, string | nil]
/// ```
///
/// ### Examples
///
/// ```lua
/// local first, second = aip.text.split_first_line("line one\nSEPARATOR\nline two", "SEPARATOR")
/// -- "line one", "line two"
///
/// local first, second = aip.text.split_first_line("line one\nNO MATCH\nline two", "SEPARATOR")
/// -- "line one\nNO MATCH\nline two", nil
///
/// local first, second = aip.text.split_first_line("SEPARATOR\nline two", "SEPARATOR")
/// -- "", "line two"
///
/// local first, second = aip.text.split_first_line("line one\nSEPARATOR", "SEPARATOR")
/// -- "line one", ""
/// ```
pub fn split_first_line(lua: &Lua, (content_val, sep_lua_str): (Value, LuaString)) -> mlua::Result<MultiValue> {
	let Some(content) = into_option_string(content_val, "aip.text.split_first_line")? else {
		return Ok(MultiValue::from_vec(vec![Value::Nil, Value::Nil]));
	};

	let sep_str = sep_lua_str.to_str()?;
	split_once_line(lua, content, &sep_str, true)
}

/// ## Lua Documentation
///
/// Splits a string into two parts based on the last line that exactly matches the separator.
/// The separator line itself is not included in either part.
/// If `content` is `nil`, returns `(nil, nil)`.
/// If no line matches the separator, returns `(original_content, nil)`.
///
/// ```lua
/// -- API Signature
/// aip.text.split_last_line(content: string | nil, sep: string): (string | nil, string | nil)
/// ```
///
/// ### Arguments
///
/// - `content: string | nil`: The string to split.
/// - `sep: string`: The exact string the line must match to be considered a separator.
///
/// ### Returns
///
/// A tuple containing the part before the separator line and the part after the separator line.
/// Empty parts are returned as empty strings.
/// Returns `(nil, nil)` if input `content` is `nil`.
/// Returns `(original_content, nil)` if no separator line is found.
///
/// ```ts
/// [string | nil, string | nil]
/// ```
///
/// ### Examples
///
/// ```lua
/// local first, second = aip.text.split_last_line("line one\nSEPARATOR\nline two\nSEPARATOR\nline three", "SEPARATOR")
/// -- "line one\nSEPARATOR\nline two", "line three"
///
/// local first, second = aip.text.split_last_line("line one\nSEPARATOR", "SEPARATOR")
/// -- "line one", ""
/// ```
pub fn split_last_line(lua: &Lua, (content_val, sep_lua_str): (Value, LuaString)) -> mlua::Result<MultiValue> {
	let Some(content) = into_option_string(content_val, "aip.text.split_last_line")? else {
		return Ok(MultiValue::from_vec(vec![Value::Nil, Value::Nil]));
	};

	let sep_str = sep_lua_str.to_str()?;
	split_once_line(lua, content, &sep_str, false)
}

/// Support function
/// `find_first` - if true, will do a split_first_line, if false, will do a split_last_line
fn split_once_line(lua: &Lua, content: String, sep: &str, find_first: bool) -> mlua::Result<MultiValue> {
	let mut match_info: Option<(usize, usize)> = None; // (start_of_sep_line_content, end_of_sep_line_content)

	let mut current_byte_offset = 0;
	for line_str in content.lines() {
		let line_content_start_idx = current_byte_offset;
		let line_content_end_idx = line_content_start_idx + line_str.len();

		if line_str == sep {
			match_info = Some((line_content_start_idx, line_content_end_idx));
			if find_first {
				break;
			}
		}
		current_byte_offset = line_content_end_idx + 1;
	}

	if let Some((sep_line_content_start_idx, sep_line_content_end_idx)) = match_info {
		let first_part_str = if sep_line_content_start_idx == 0 {
			""
		} else {
			&content[0..sep_line_content_start_idx.saturating_sub(1)]
		};

		let second_part_start_idx = sep_line_content_end_idx + 1;
		let second_part_str = if second_part_start_idx > content.len() {
			""
		} else {
			&content[second_part_start_idx..]
		};

		Ok(MultiValue::from_vec(vec![
			Value::String(lua.create_string(first_part_str)?),
			Value::String(lua.create_string(second_part_str)?),
		]))
	} else {
		// No match found
		Ok(MultiValue::from_vec(vec![
			Value::String(lua.create_string(&content)?),
			Value::Nil,
		]))
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_text;
	use serde_json::Value as JsonValue;

	// region:    --- Support

	fn get_returned_parts(res: JsonValue) -> Result<(Option<String>, Option<String>)> {
		let arr = res.as_array().ok_or("Result should be an array")?;
		let first_json = arr.first().ok_or("Missing first part")?;
		let second = arr
			.get(1)
			.map(|v| v.as_str().map(|s| s.to_string()).ok_or("Should be string"))
			.transpose()?;

		let first = if first_json.is_null() {
			None
		} else {
			Some(first_json.as_str().ok_or("First part not string")?.to_string())
		};

		Ok((first, second))
	}

	// endregion: --- Support

	#[tokio::test]
	async fn test_lua_text_split_first_line_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let test_cases = vec![
			// content, separator, expected_first, expected_second
			("line1\nSEP\nline2", "SEP", Some("line1"), Some("line2")),
			("SEP\nline2", "SEP", Some(""), Some("line2")),
			("line1\nSEP", "SEP", Some("line1"), Some("")),
			("SEP", "SEP", Some(""), Some("")),
			("line1\nNO_MATCH\nline2", "SEP", Some("line1\nNO_MATCH\nline2"), None),
			(
				"line1\nSEP\nline2\nSEP\nline3",
				"SEP",
				Some("line1"),
				Some("line2\nSEP\nline3"),
			),
			("", "SEP", Some(""), None),                          // Empty content
			("line1\n\nline2", "", Some("line1"), Some("line2")), // Empty line as separator
			("\nline2", "", Some(""), Some("line2")),             // Separator is first line (empty)
			("line1\n", "", Some("line1\n"), None),               // Should be none
			("text without newlines", "SEP", Some("text without newlines"), None),
			("SEP_LINE_NO_NEWLINE_END", "SEP_LINE_NO_NEWLINE_END", Some(""), Some("")),
		];

		// -- Exec & Check
		for (content_str, sep_str, exp_first_opt, exp_second_opt) in test_cases {
			let script = format!(
				r#"
                local content = {content_str:?}
                local sep = {sep_str:?}
                local first, second = aip.text.split_first_line(content, sep)
                return {{first, second}}
                "#,
			);
			let res_json = eval_lua(&lua, &script)?;
			let (first, second) = get_returned_parts(res_json)?;

			assert_eq!(
				first,
				exp_first_opt.map(String::from),
				"Content: {content_str}, Sep: {sep_str}"
			);
			assert_eq!(
				second,
				exp_second_opt.map(String::from),
				"Content: {content_str}, Sep: {sep_str}"
			);
		}
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_split_first_line_nil_content() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let script = r#"
        local first, second = aip.text.split_first_line(nil, "SEP")
        return {first, second}
        "#;

		// -- Exec
		let res_json = eval_lua(&lua, script)?;

		// -- Check
		// This will be empty object (lua {nil, nil, ...} all nil is empty object)
		assert!(
			res_json.as_object().ok_or("Should be object")?.is_empty(),
			"Should be empty"
		);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_split_last_line_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let test_cases = vec![
			// content, separator, expected_first, expected_second
			("line1\nSEP\nline2", "SEP", Some("line1"), Some("line2")),
			("SEP\nline2", "SEP", Some(""), Some("line2")),
			("line1\nSEP", "SEP", Some("line1"), Some("")),
			("SEP", "SEP", Some(""), Some("")),
			("line1\nNO_MATCH\nline2", "SEP", Some("line1\nNO_MATCH\nline2"), None),
			(
				"line1\nSEP\nline2\nSEP\nline3",
				"SEP",
				Some("line1\nSEP\nline2"),
				Some("line3"),
			),
			("SEP\nmiddle\nSEP", "SEP", Some("SEP\nmiddle"), Some("")),
			("", "SEP", Some(""), None),                          // Empty content
			("line1\n\nline2", "", Some("line1"), Some("line2")), // Empty line as separator
			("\nline2", "", Some(""), Some("line2")),
			("line1\n", "", Some("line1\n"), None),
			("text without newlines", "SEP", Some("text without newlines"), None),
			("SEP_LINE_NO_NEWLINE_END", "SEP_LINE_NO_NEWLINE_END", Some(""), Some("")),
		];

		// -- Exec & Check
		for (content_str, sep_str, exp_first_opt, exp_second_opt) in test_cases {
			let script = format!(
				r#"
                local content = {content_str:?}
                local sep = {sep_str:?}
                local first, second = aip.text.split_last_line(content, sep)
                return {{first, second}}
                "#,
			);
			let res_json = eval_lua(&lua, &script)?;
			let (first, second) = get_returned_parts(res_json)?;

			assert_eq!(
				first,
				exp_first_opt.map(String::from),
				"Content: {content_str:?}, Sep: {sep_str:?}"
			);
			assert_eq!(
				second,
				exp_second_opt.map(String::from),
				"Content: {content_str:?}, Sep: {sep_str:?}"
			);
		}
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_split_last_line_nil_content() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let script = r#"
        local first, second = aip.text.split_last_line(nil, "SEP")
        return {first, second}
        "#;

		// -- Exec
		let res_json = eval_lua(&lua, script)?;

		// -- Check
		// This will be empty object (lua {nil, nil, ...} all nil is empty object)
		assert!(
			res_json.as_object().ok_or("Should be object")?.is_empty(),
			"Should be empty"
		);

		Ok(())
	}
}

// endregion: --- Tests
