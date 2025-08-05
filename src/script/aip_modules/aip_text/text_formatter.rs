// region:    --- Formatters

use crate::script::LuaValueExt as _;
use mlua::{Lua, Value};
use simple_fs::PrettySizeOptions;

/// ## Lua Documentation
///
/// Formats a byte count into a human-readable 9 characters right aligned string
///
/// `777`           -> `"   777 B "`
/// `8777`          -> `"  8.78 KB"`
/// `88777`         -> `" 88.78 KB"`
/// `888777`        -> `"888.78 KB"`
/// `2_345_678_900` -> `"  2.35 GB"`
///
/// ```lua
/// -- API Signature
/// aip.text.format_size(bytes: integer | nil): string | nil
/// aip.text.format_size(bytes: integer | nil, options: string | {lowest_unit: string, trim: boolean} | nil): string | nil
/// ```
///
/// ### Arguments
///
/// - `bytes: integer | nil`: The number of bytes to format.  
///   Accepts both Lua `integer` and `number` (floats are truncated).
/// - `options: string | {lowest_unit: string, trim: boolean} | nil`:
///   Either a string representing the lowest unit (like "MB"), or a table with:
///   - `lowest_unit: string` (optional): The lowest unit to display (e.g.,"MB")
///   - `trim: boolean` (optional): Whether to trim whitespace from the result (default: false)
///
/// ### Returns
///
/// A formatted size string, or `nil` if `bytes` is `nil`.
pub fn format_size(lua: &Lua, (bytes_val, options): (Value, Option<Value>)) -> mlua::Result<Value> {
	let bytes: u64 = match bytes_val {
		Value::Nil => return Ok(Value::Nil),
		Value::Integer(i) => i.max(0) as u64,
		Value::Number(n) => n.round().max(0.0) as u64,
		other => {
			return Err(mlua::Error::FromLuaConversionError {
				from: other.type_name(),
				to: "integer".to_string(),
				message: Some("bytes argument must be a number".into()),
			});
		}
	};

	let format_options = FormatSizeOptions::new(options);
	let pretty_options = format_options.lowest_unit.as_deref().map(PrettySizeOptions::from);

	let pretty = crate::support::text::format_pretty_size(bytes, pretty_options);
	let result = if format_options.trim {
		pretty.trim().to_string()
	} else {
		pretty
	};
	lua.create_string(&result).map(Value::String)
}

#[derive(Debug, Default)]
struct FormatSizeOptions {
	lowest_unit: Option<String>,
	trim: bool,
}

impl FormatSizeOptions {
	fn new(val: Option<Value>) -> Self {
		let Some(val) = val else {
			return Default::default();
		};

		// If it's a string, treat it as lowest_unit
		if let Some(s) = val.x_as_lua_str() {
			return FormatSizeOptions {
				lowest_unit: Some(s.to_string()),
				trim: false,
			};
		}

		// If it's a table, extract properties
		if let Some(table) = val.as_table() {
			let lowest_unit = table.x_get_string("lowest_unit");
			let trim = table.x_get_bool("trim").unwrap_or(false);

			return FormatSizeOptions { lowest_unit, trim };
		}

		// If it's neither string nor table, return default
		Default::default()
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_text;
	use crate::support::text::format_pretty_size;

	#[tokio::test]
	async fn test_lua_text_format_size_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let test_vals = [0_i64, 1500, 5_242_880];

		// -- Exec & Check
		for &bytes in &test_vals {
			let script = format!("return aip.text.format_size({bytes})");
			let res = eval_lua(&lua, &script)?;
			let lua_str = res.as_str().ok_or("Should be string")?;
			let expected = format_pretty_size(bytes as u64, None);
			assert_eq!(lua_str, expected);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_format_size_nil() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;
		let script = r#"return aip.text.format_size(nil)"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert!(res.is_null(), "Expected nil return for nil input");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_format_size_with_options_string() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;

		// -- Exec & Check
		let script = r#"return aip.text.format_size(1500, "KB")"#;
		let res = eval_lua(&lua, script)?;
		let lua_str = res.as_str().ok_or("Should be string")?;
		let expected = format_pretty_size(1500, Some(simple_fs::PrettySizeOptions::from("KB")));
		assert_eq!(lua_str, expected);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_format_size_with_options_table() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;

		// -- Exec & Check
		let script = r#"return aip.text.format_size(1500, {lowest_unit = "KB", trim = true})"#;
		let res = eval_lua(&lua, script)?;
		let lua_str = res.as_str().ok_or("Should be string")?;
		let expected = format_pretty_size(1500, Some(simple_fs::PrettySizeOptions::from("KB")));
		assert_eq!(lua_str, expected.trim());

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_format_size_with_options_table_default_trim() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text").await?;

		// -- Exec & Check
		let script = r#"return aip.text.format_size(1500, {lowest_unit = "KB"})"#;
		let res = eval_lua(&lua, script)?;
		let lua_str = res.as_str().ok_or("Should be string")?;
		let expected = format_pretty_size(1500, Some(simple_fs::PrettySizeOptions::from("KB")));
		assert_eq!(lua_str, expected);

		Ok(())
	}
}

// endregion: --- Tests

// endregion: --- Formatters
