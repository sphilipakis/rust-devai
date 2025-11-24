//! Defines the `lua` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//! The `lua` module exposes functions for inspecting and dumping Lua values.
//!
//! ### Functions
//!
//! - `aip.lua.dump(value: any) -> string`

use crate::Result;
use crate::runtime::Runtime;
use crate::script::NullSentinel;
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let dump_lua = lua.create_function(dump)?;
	table.set("dump", dump_lua)?;

	let merge_lua = lua.create_function(merge)?;
	table.set("merge", merge_lua)?;

	let merge_deep_lua = lua.create_function(merge_deep)?;
	table.set("merge_deep", merge_deep_lua)?;

	Ok(table)
}

// region: --- Rust Lua Support

/// ## Lua Documentation
///
/// Shallow merge of `overlay` into `base`. Returns a new table.
///
/// ```lua
/// -- API Signature
/// aip.lua.merge(base: table, overlay: table): table
/// ```
fn merge(lua: &Lua, (base, overlay): (Table, Table)) -> mlua::Result<Table> {
	let new_table = lua.create_table()?;
	for pair in base.pairs::<Value, Value>() {
		let (k, v) = pair?;
		new_table.set(k, v)?;
	}
	for pair in overlay.pairs::<Value, Value>() {
		let (k, v) = pair?;
		new_table.set(k, v)?;
	}
	Ok(new_table)
}

/// ## Lua Documentation
///
/// Deep merge of `overlay` into `base`. Returns a new table.
///
/// ```lua
/// -- API Signature
/// aip.lua.merge_deep(base: table, overlay: table): table
/// ```
fn merge_deep(lua: &Lua, (base, overlay): (Table, Table)) -> mlua::Result<Table> {
	fn deep_merge_value(lua: &Lua, base: Value, overlay: Value) -> mlua::Result<Value> {
		match (base, overlay) {
			(Value::Table(base_t), Value::Table(overlay_t)) => {
				let new_table = lua.create_table()?;
				// Copy base
				for pair in base_t.pairs::<Value, Value>() {
					let (k, v) = pair?;
					new_table.set(k, v)?;
				}
				// Merge overlay
				for pair in overlay_t.pairs::<Value, Value>() {
					let (k, v_overlay) = pair?;
					let v_base = new_table.get::<Value>(k.clone())?;
					let merged = deep_merge_value(lua, v_base, v_overlay)?;
					new_table.set(k, merged)?;
				}
				Ok(Value::Table(new_table))
			}
			(_, v) => Ok(v),
		}
	}

	let res = deep_merge_value(lua, Value::Table(base), Value::Table(overlay))?;
	match res {
		Value::Table(t) => Ok(t),
		_ => unreachable!("Should be a table"),
	}
}

/// ## Lua Documentation
///
/// Dump a Lua value into its string representation.
///
/// ```lua
/// -- API Signature
/// aip.lua.dump(value: any): string
/// ```
///
/// Given any Lua value, returns a string that recursively represents tables and their structure.
/// Useful for debugging and logging purposes.
///
/// ### Arguments
///
/// - `value`: The Lua value to be dumped. Can be any Lua type (nil, boolean, number, string, table, function, userdata, etc.).
///
/// ### Returns
///
/// - `string`: A string representation of the Lua value. Table contents are recursively dumped with indentation. Functions, userdata, and threads are represented by placeholder strings.
///
/// ```ts
/// string
/// ```
///
/// ### Example
///
/// ```lua
/// local tbl = { key = "value", nested = { subkey = 42 } }
/// print(aip.lua.dump(tbl))
/// -- Expected output structure (exact indentation/formatting might vary slightly based on internal Lua/dump implementation):
/// -- {
/// --   nested = {
/// --     key1 = "value1",
/// --     key2 = 42
/// --   },
/// --   bool = true,
/// --   num = 3.21
/// -- }
///
/// print(aip.lua.dump("Hello World"))
/// -- Expected output: "Hello World"
///
/// print(aip.lua.dump(123.45))
/// -- Expected output: 123.45
/// ```
///
/// ### Error
///
/// Returns an error message string if the conversion of a value (like a non-UTF8 string or specific userdata) within the dump process fails.
///
/// ```ts
/// {
///   error: string // Error message detailing the conversion failure
/// }
/// ```
pub fn dump(lua: &Lua, value: Value) -> mlua::Result<String> {
	fn dump_value(_lua: &Lua, value: Value, indent: usize) -> mlua::Result<String> {
		let indent_str = "  ".repeat(indent);
		match value {
			Value::Nil => Ok("nil".to_string()),
			Value::Boolean(b) => Ok(b.to_string()),
			Value::Integer(i) => Ok(i.to_string()),
			Value::Number(n) => Ok(n.to_string()),
			Value::String(s) => {
				let s: String = s.to_str()?.to_string();
				Ok(format!("\"{s}\""))
			}
			Value::Table(t) => {
				let mut entries: Vec<String> = Vec::new();
				for pair in t.clone().pairs::<Value, Value>() {
					let (key, val) = pair?;
					let dumped_key = match key {
						Value::String(s) => s.to_str()?.to_string(),
						_ => dump_value(_lua, key, 0)?,
					};
					let dumped_val = dump_value(_lua, val, indent + 1)?;
					entries.push(format!(
						"{indent_str_for_entry}{dumped_key} = {dumped_val}",
						indent_str_for_entry = "  ".repeat(indent + 1)
					));
				}
				let inner = entries.join(",\n");
				Ok(format!("{{\n{inner}\n{indent_str}}}"))
			}
			Value::Function(f) => {
				let name = f.info().name.unwrap_or("<anonymous>".to_string());
				Ok(format!("<function {name}>"))
			}
			Value::UserData(ud) => {
				if let Ok(ns) = ud.borrow::<NullSentinel>() {
					Ok(ns.to_string())
				} else {
					Ok("<UserData>".into())
				}
			}
			Value::LightUserData(_) => Ok("<LightUserData>".to_string()),
			Value::Thread(_) => Ok("<Thread>".to_string()),
			_ => Ok("<OtherType>".to_string()),
		}
	}

	dump_value(lua, value, 0)
}
// endregion: --- Rust Lua Support

// region: --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules::aip_lua;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_lua_dump_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_lua::init_module, "lua").await?;
		let script = r#"
local tbl = {
  nested = { key1 = "value1", key2 = 42 },
  bool = true,
  num = 3.21
}
return aip.lua.dump(tbl)
	    "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;
		let res = res.as_str().ok_or("res json value should be of type string")?;

		// -- Check
		assert_contains(res, "bool = true");
		assert_contains(res, "key1 = \"value1\"");
		assert_contains(res, "key2 = 42");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_lua_merge_simple_ok() -> Result<()> {
		// -- Setup
		let lua = setup_lua(aip_lua::init_module, "lua").await?;

		// -- Test shallow merge
		let script = r#"
        local base = { a = 1, b = 2 }
        local ovl = { b = 3, c = 4 }
        local res = aip.lua.merge(base, ovl)
        return res
    "#;
		let res = eval_lua(&lua, script)?;
		assert_eq!(res.x_get::<i32>("a")?, 1);
		assert_eq!(res.x_get::<i32>("b")?, 3);
		assert_eq!(res.x_get::<i32>("c")?, 4);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_lua_merge_deep_ok() -> Result<()> {
		// -- Setup
		let lua = setup_lua(aip_lua::init_module, "lua").await?;

		// -- Test deep merge
		let script = r#"
        local base = { a = 1, b = { x = 10, y = 20 } }
        local ovl = { b = { y = 22, z = 30 }, c = 3 }
        local res = aip.lua.merge_deep(base, ovl)
        return res
    "#;
		let res = eval_lua(&lua, script)?;
		assert_eq!(res.x_get::<i32>("a")?, 1);
		assert_eq!(res.x_get::<i32>("c")?, 3);

		assert_eq!(res.x_get::<i32>("/b/x")?, 10);
		assert_eq!(res.x_get::<i32>("/b/y")?, 22);
		assert_eq!(res.x_get::<i32>("/b/z")?, 30);

		Ok(())
	}
}
// endregion: --- Tests
