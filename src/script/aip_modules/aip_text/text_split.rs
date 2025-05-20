//! Defines text splitting functions for the `aip.text` Lua module.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This section of the `aip.text` module exposes functions for splitting text.
//!
//! ### Functions
//!
//! - `aip.text.split_first(content: string | nil, sep: string): (string | nil, string | nil)`
//! - `aip.text.split_last(content: string | nil, sep: string): (string | nil, string | nil)`

use crate::script::support::into_option_string;
use mlua::{Lua, MultiValue, String as LuaString, Value};

/// ## Lua Documentation
///
/// Splits a string into two parts based on the first occurrence of a separator.
/// If `content` is `nil`, returns `(nil, nil)`.
///
/// ```lua
/// -- API Signature
/// aip.text.split_first(content: string | nil, sep: string): (string | nil, string | nil)
/// ```
///
/// ### Arguments
///
/// - `content: string | nil`: The string to split.
/// - `sep: string`: The separator string.
///
/// ### Returns
///
/// A tuple containing the first part and the second part (or nil if no match).
/// Returns `(nil, nil)` if input `content` is `nil`.
///
/// ```ts
/// [string | nil, string | nil]
/// ```
///
/// ### Examples
///
/// ```lua
/// local first, second = aip.text.split_first("some == text", "==")
/// -- "some ", " text"
///
/// local first, second = aip.text.split_first("some == text", "++")
/// -- "some == text", nil
/// ```
pub fn split_first(lua: &Lua, (content_val, sep_lua_str): (Value, LuaString)) -> mlua::Result<MultiValue> {
	let Some(content) = into_option_string(content_val, "aip.text.split_first")? else {
		return Ok(MultiValue::from_vec(vec![Value::Nil, Value::Nil]));
	};

	let sep_str = sep_lua_str.to_str()?;

	split_once(lua, content, &sep_str, true)
}

/// ## Lua Documentation
///
/// Splits a string into two parts based on the last occurrence of a separator.
/// If `content` is `nil`, returns `(nil, nil)`.
///
/// ```lua
/// -- API Signature
/// aip.text.split_last(content: string | nil, sep: string): (string | nil, string | nil)
/// ```
///
/// ### Arguments
///
/// - `content: string | nil`: The string to split.
/// - `sep: string`: The separator string.
///
/// ### Returns
///
/// A tuple containing the first part and the second part (or nil if no match).
/// Returns `(nil, nil)` if input `content` is `nil`.
///
/// ```ts
/// [string | nil, string | nil]
/// ```
///
/// ### Examples
///
/// ```lua
/// local first, second = aip.text.split_last("some == text == more", "==")
/// -- "some == text ", " more"
///
/// local first, second = aip.text.split_last("some == text", "++")
/// -- "some == text", nil
/// ```
pub fn split_last(lua: &Lua, (content_val, sep_lua_str): (Value, LuaString)) -> mlua::Result<MultiValue> {
	let Some(content) = into_option_string(content_val, "aip.text.split_last")? else {
		return Ok(MultiValue::from_vec(vec![Value::Nil, Value::Nil]));
	};

	let sep_str = sep_lua_str.to_str()?;

	split_once(lua, content, &sep_str, false)
}

/// Support function
/// `first` - if true, will do a split_first, if false, will do a split_last
fn split_once(lua: &Lua, content: String, sep: &str, first: bool) -> mlua::Result<MultiValue> {
	let index = if first { content.find(sep) } else { content.rfind(sep) };

	if let Some(index) = index {
		let first_part = &content[..index];
		let second_part = &content[index + sep.len()..];

		Ok(MultiValue::from_vec(vec![
			Value::String(lua.create_string(first_part)?),
			Value::String(lua.create_string(second_part)?),
		]))
	} else {
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

	#[tokio::test]
	async fn test_lua_text_split_first_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text")?;
		// (content, separator, (first, second))
		let data = [
			// with matching
			(
				"some first content\n===\nsecond content",
				"===",
				("some first content\n", Some("\nsecond content")),
			),
			// no matching
			("some first content\n", "===", ("some first content\n", None)),
			// matching but nothing after separator
			("some first content\n===", "===", ("some first content\n", Some(""))),
		];

		// -- Exec & Check
		for (content, sep, expected) in data {
			let script = format!(
				r#"
			local first, second = aip.text.split_first({content:?}, "{sep}")
			return {{first, second}}
			"#
			);
			let res = eval_lua(&lua, &script)?;

			// -- Check
			let values = res.as_array().ok_or("Should have returned an array")?;

			let first = values
				.first()
				.ok_or("Should always have at least a first return")?
				.as_str()
				.ok_or("First should be string")?;
			assert_eq!(expected.0, first);

			let second_val = values.get(1);
			if let Some(exp_second) = expected.1 {
				let second_val = second_val.ok_or("Should have at least one")?; // if expected.1 is Some, this should exist
				assert_eq!(exp_second, second_val.as_str().ok_or("Should be string")?);
			} else {
				assert!(second_val.is_none(), "Second should have been none");
			}
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_split_first_nil_content() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text")?;
		let script = r#"
        local first, second = aip.text.split_first(nil, "===")
        return {first, second}
    "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		// NOTE: Because it returns {nil, nil}, then the json ignore the nil, and we have empty json object (since lua have one constructs for both)
		let res = res.as_object().ok_or("Should be object")?;
		assert!(res.is_empty(), "Should be empty");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_split_last_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text")?;
		// (content, separator, (first, second))
		let data = [
			// with matching
			("some == text == more", "==", ("some == text ", Some(" more"))),
			// with multiple matching and newlines
			(
				"line1\n==\nline2\n==\nline3",
				"==",
				("line1\n==\nline2\n", Some("\nline3")),
			),
			// no matching
			("some first content\n", "===", ("some first content\n", None)),
			// matching but nothing after separator
			("some first content\n===", "===", ("some first content\n", Some(""))),
		];

		// -- Exec & Check
		for (content, sep, expected) in data {
			let script = format!(
				r#"
			local first, second = aip.text.split_last({content:?}, "{sep}")
			return {{first, second}}
			"#
			);
			let res = eval_lua(&lua, &script)?;

			// -- Check
			let values = res.as_array().ok_or("Should have returned an array")?;

			let first = values
				.first()
				.ok_or("Should always have at least a first return")?
				.as_str()
				.ok_or("First should be string")?;
			assert_eq!(expected.0, first);

			let second_val = values.get(1);
			if let Some(exp_second) = expected.1 {
				let second_val = second_val.ok_or("Should have at least one")?; // if expected.1 is Some, this should exist
				assert_eq!(exp_second, second_val.as_str().ok_or("Should be string")?);
			} else {
				assert!(second_val.is_none(), "Second should have been none");
			}
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_split_last_nil_content() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text")?;
		let script = r#"
        local first, second = aip.text.split_last(nil, "===")
        return {first, second}
    "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		// NOTE: Because it returns {nil, nil}, then the json ignore the nil, and we have empty json object (since lua have one constructs for both)
		let res = res.as_object().ok_or("Should be object")?;
		assert!(res.is_empty(), "Should be empty");
		Ok(())
	}
}

// endregion: --- Tests
