//! Defines text trim functions for the `aip.text` Lua module.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This section of the `aip.text` module exposes functions for trimming text content.
//!
//! ### Functions
//!
//! - `aip.text.trim(content: string | nil): string | nil`
//! - `aip.text.trim_start(content: string | nil): string | nil`
//! - `aip.text.trim_end(content: string | nil): string | nil`

use crate::script::support::into_option_string;
use mlua::{Lua, Value};

/// ## Lua Documentation
///
/// Trims leading and trailing whitespace from a string.
/// If `content` is `nil`, returns `nil`.
///
/// ```lua
/// -- API Signature
/// aip.text.trim(content: string | nil): string | nil
/// ```
///
/// ### Arguments
///
/// - `content: string | nil`: The string to trim.
///
/// ### Returns
///
/// The trimmed string, or `nil` if input `content` is `nil`.
pub fn trim(lua: &Lua, content_val: Value) -> mlua::Result<Value> {
	let Some(content_str) = into_option_string(content_val, "aip.text.trim")? else {
		return Ok(Value::Nil);
	};
	let trimmed_str = content_str.trim();
	lua.create_string(trimmed_str).map(Value::String)
}

/// ## Lua Documentation
///
/// Trims leading whitespace from a string.
/// If `content` is `nil`, returns `nil`.
///
/// ```lua
/// -- API Signature
/// aip.text.trim_start(content: string | nil): string | nil
/// ```
///
/// ### Arguments
///
/// - `content: string | nil`: The string to trim.
///
/// ### Returns
///
/// The trimmed string, or `nil` if input `content` is `nil`.
pub fn trim_start(lua: &Lua, content_val: Value) -> mlua::Result<Value> {
	let Some(content_str) = into_option_string(content_val, "aip.text.trim_start")? else {
		return Ok(Value::Nil);
	};
	let trimmed_str = content_str.trim_start();
	lua.create_string(trimmed_str).map(Value::String)
}

/// ## Lua Documentation
///
/// Trims trailing whitespace from a string.
/// If `content` is `nil`, returns `nil`.
///
/// ```lua
/// -- API Signature
/// aip.text.trim_end(content: string | nil): string | nil
/// ```
///
/// ### Arguments
///
/// - `content: string | nil`: The string to trim.
///
/// ### Returns
///
/// The trimmed string, or `nil` if input `content` is `nil`.
pub fn trim_end(lua: &Lua, content_val: Value) -> mlua::Result<Value> {
	let Some(content_str) = into_option_string(content_val, "aip.text.trim_end")? else {
		return Ok(Value::Nil);
	};
	let trimmed_str = content_str.trim_end();
	lua.create_string(trimmed_str).map(Value::String)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_text;

	#[tokio::test]
	async fn test_lua_text_trim_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let script = r#"return aip.text.trim("  hello world  ")"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Should be string")?, "hello world");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_trim_nil_content() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let script = r#"return aip.text.trim(nil)"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert!(res.is_null(), "Expected null for nil content input");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_trim_start_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let script = r#"return aip.text.trim_start("  hello world  ")"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Should be string")?, "hello world  ");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_trim_start_nil_content() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let script = r#"return aip.text.trim_start(nil)"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert!(res.is_null(), "Expected null for nil content input");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_trim_end_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let script = r#"return aip.text.trim_end("  hello world  ")"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Should be string")?, "  hello world");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_trim_end_nil_content() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let script = r#"return aip.text.trim_end(nil)"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert!(res.is_null(), "Expected null for nil content input");
		Ok(())
	}
}

// endregion: --- Tests
