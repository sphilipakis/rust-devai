//! Defines key-focused helpers for the `aip.shape` Lua module.
//!
//! ## Lua documentation
//!
//! This section of the `aip.shape` module exposes helpers to select, omit, remove, or extract keys on row-like records (Lua objects).
//!
//! ### Functions
//!
//! - `aip.shape.select_keys(rec: table, keys: string[]): table`
//!
//! - `aip.shape.omit_keys(rec: table, keys: string[]): table`
//!
//! - `aip.shape.remove_keys(rec: table, keys: string[]): integer`
//!
//! - `aip.shape.extract_keys(rec: table, keys: string[]): table`

use crate::script::support::{collect_string_sequence, expect_table};
use mlua::{Lua, Value};

/// ## Lua Documentation
/// ---
/// Return a new record containing only the specified keys. The original record is not modified.
///
/// ```lua
/// -- API Signature
/// aip.shape.select_keys(rec: object, keys: string[]): object
/// ```
///
/// - Missing keys are ignored.
/// - If `keys` contains a non-string entry, an error is returned.
///
/// ### Example
/// ```lua
/// local rec  = { id = 1, name = "Alice", email = "a@x.com" }
/// local keys = { "id", "email" }
/// local out  = aip.shape.select_keys(rec, keys)
/// -- out == { id = 1, email = "a@x.com" }
/// ```
pub fn select_keys(lua: &Lua, rec: Value, keys: Value) -> mlua::Result<Value> {
	let rec = expect_table(rec, "aip.shape.select_keys", "Record")?;
	let key_names = collect_string_sequence(keys, "aip.shape.select_keys", "Key names")?;
	let out = lua.create_table()?;

	for key in key_names {
		let val: Value = rec.get(key.clone())?;
		if !val.is_nil() {
			out.set(key, val)?;
		}
	}

	Ok(Value::Table(out))
}

///
/// Return a new record without the specified keys. The original record is not modified.
///
/// - Missing keys are ignored.
/// - If `keys` contains a non-string entry, an error is returned.
///
/// ## Lua Documentation
/// ---
/// Return a new record without the specified keys. The original record is not modified.
///
/// ```lua
/// -- API Signature
/// aip.shape.omit_keys(rec: object, keys: string[]): object
/// ```
///
/// ### Example
/// ```lua
/// local rec  = { id = 1, name = "Alice", email = "a@x.com" }
/// local out  = aip.shape.omit_keys(rec, { "email" })
/// -- out == { id = 1, name = "Alice" }
/// -- rec is unchanged
/// ```
pub fn omit_keys(lua: &Lua, rec: Value, keys: Value) -> mlua::Result<Value> {
	let rec = expect_table(rec, "aip.shape.omit_keys", "Record")?;
	use std::collections::HashSet;

	let key_names = collect_string_sequence(keys, "aip.shape.omit_keys", "Key names")?;
	let mut omit_set: HashSet<String> = HashSet::with_capacity(key_names.len());
	for key in key_names {
		omit_set.insert(key.to_string_lossy().to_string());
	}

	let out = lua.create_table()?;
	for pair in rec.pairs::<Value, Value>() {
		let (k, v) = pair?;
		let skip = match &k {
			Value::String(s) => {
				let needle = s.to_string_lossy();
				omit_set.contains(&needle)
			}
			_ => false,
		};
		if !skip {
			out.set(k, v)?;
		}
	}

	Ok(Value::Table(out))
}

///
/// Remove the specified keys from the original record (in-place mutation) and return
/// the number of keys actually removed.
///
/// - Missing keys are ignored.
/// - If `keys` contains a non-string entry, an error is returned.
///
/// ## Lua Documentation
/// ---
/// Remove the specified keys from the original record (in-place) and return
/// the number of keys actually removed.
///
/// ```lua
/// -- API Signature
/// aip.shape.remove_keys(rec: object, keys: string[]): integer
/// ```
///
/// ### Example
/// ```lua
/// local rec = { id = 1, name = "Alice", email = "a@x.com" }
/// local n   = aip.shape.remove_keys(rec, { "email", "missing" })
/// -- n   == 1
/// -- rec == { id = 1, name = "Alice" }
/// ```
pub fn remove_keys(_lua: &Lua, rec: Value, keys: Value) -> mlua::Result<Value> {
	let rec = expect_table(rec, "aip.shape.remove_keys", "Record")?;
	let key_names = collect_string_sequence(keys, "aip.shape.remove_keys", "Key names")?;
	let mut removed: i64 = 0;

	for key in key_names {
		let val: Value = rec.get(key.clone())?;
		if !val.is_nil() {
			rec.set(key, Value::Nil)?;
			removed += 1;
		}
	}

	Ok(Value::Integer(removed))
}

///
/// Return a new record containing only the specified keys and remove them from the original record (in-place).
///
/// - Missing keys are ignored.
/// - If `keys` contains a non-string entry, an error is returned.
///
/// ## Lua Documentation
/// ---
/// Return a new record containing only the specified keys and remove them from the original record (in-place).
///
/// ```lua
/// -- API Signature
/// aip.shape.extract_keys(rec: object, keys: string[]): object
/// ```
///
/// ### Example
/// ```lua
/// local rec     = { id = 1, name = "Alice", email = "a@x.com" }
/// local picked  = aip.shape.extract_keys(rec, { "id", "email" })
/// -- picked == { id = 1, email = "a@x.com" }
/// -- rec    == { name = "Alice" }
/// ```
pub fn extract_keys(lua: &Lua, rec: Value, keys: Value) -> mlua::Result<Value> {
	let rec = expect_table(rec, "aip.shape.extract_keys", "Record")?;
	let key_names = collect_string_sequence(keys, "aip.shape.extract_keys", "Key names")?;
	let out = lua.create_table()?;

	for key in key_names {
		let val: Value = rec.get(key.clone())?;
		if !val.is_nil() {
			out.set(key.clone(), val)?;
			// Remove from original record (in-place)
			rec.set(key, Value::Nil)?;
		}
	}

	Ok(Value::Table(out))
}

// region:    --- Tests

#[cfg(test)]
#[path = "shape_keys_tests.rs"]
mod tests;
// endregion: --- Tests
