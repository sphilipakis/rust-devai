// region:    --- Formatters

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
/// ```
///
/// ### Arguments
///
/// - `bytes: integer | nil`: The number of bytes to format.  
///   Accepts both Lua `integer` and `number` (floats are truncated).
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
	let options = if let Some(options) = options
		&& let Some(lowest_unit) = options.as_string()
	{
		PrettySizeOptions::from(lowest_unit.to_string_lossy())
	} else {
		PrettySizeOptions::default()
	};

	let pretty = crate::support::text::format_pretty_size(bytes, Some(options));
	lua.create_string(&pretty).map(Value::String)
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
}

// endregion: --- Tests

// endregion: --- Formatters
