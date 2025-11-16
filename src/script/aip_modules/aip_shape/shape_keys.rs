//! Defines key-focused helpers for the `aip.shape` Lua module.
//!
//! ## Lua documentation
//!
//! This section of the `aip.shape` module exposes helpers to select, omit, remove, or extract keys on row-like records (Lua tables).
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

use crate::Error;
use mlua::{Lua, Table, Value};

/// ## Lua Documentation
/// ---
/// Return a new record containing only the specified keys. The original record is not modified.
///
/// ```lua
/// -- API Signature
/// aip.shape.select_keys(rec: table, keys: string[]): table
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
pub fn select_keys(lua: &Lua, rec: Table, keys: Table) -> mlua::Result<Value> {
	let out = lua.create_table()?;

	for (idx, key_val) in keys.sequence_values::<Value>().enumerate() {
		let key_val = key_val?;
		let key_str = match key_val {
			Value::String(s) => s,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.select_keys - Key names must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		};

		let val: Value = rec.get(key_str.clone())?;
		if !val.is_nil() {
			out.set(key_str, val)?;
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
/// aip.shape.omit_keys(rec: table, keys: string[]): table
/// ```
///
/// ### Example
/// ```lua
/// local rec  = { id = 1, name = "Alice", email = "a@x.com" }
/// local out  = aip.shape.omit_keys(rec, { "email" })
/// -- out == { id = 1, name = "Alice" }
/// -- rec is unchanged
/// ```
pub fn omit_keys(lua: &Lua, rec: Table, keys: Table) -> mlua::Result<Value> {
	use std::collections::HashSet;

	let mut omit_set: HashSet<String> = HashSet::new();
	for (idx, key_val) in keys.sequence_values::<Value>().enumerate() {
		let key_val = key_val?;
		match key_val {
			Value::String(s) => {
				omit_set.insert(s.to_string_lossy());
			}
			other => {
				return Err(Error::custom(format!(
					"aip.shape.omit_keys - Key names must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		}
	}

	let out = lua.create_table()?;
	for pair in rec.pairs::<Value, Value>() {
		let (k, v) = pair?;
		let skip = match &k {
			Value::String(s) => omit_set.contains(&s.to_string_lossy()),
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
/// aip.shape.remove_keys(rec: table, keys: string[]): integer
/// ```
///
/// ### Example
/// ```lua
/// local rec = { id = 1, name = "Alice", email = "a@x.com" }
/// local n   = aip.shape.remove_keys(rec, { "email", "missing" })
/// -- n   == 1
/// -- rec == { id = 1, name = "Alice" }
/// ```
pub fn remove_keys(_lua: &Lua, rec: Table, keys: Table) -> mlua::Result<Value> {
	let mut removed: i64 = 0;

	for (idx, key_val) in keys.sequence_values::<Value>().enumerate() {
		let key_val = key_val?;
		let key_str = match key_val {
			Value::String(s) => s,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.remove_keys - Key names must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		};

		let val: Value = rec.get(key_str.clone())?;
		if !val.is_nil() {
			rec.set(key_str, Value::Nil)?;
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
/// aip.shape.extract_keys(rec: table, keys: string[]): table
/// ```
///
/// ### Example
/// ```lua
/// local rec     = { id = 1, name = "Alice", email = "a@x.com" }
/// local picked  = aip.shape.extract_keys(rec, { "id", "email" })
/// -- picked == { id = 1, email = "a@x.com" }
/// -- rec    == { name = "Alice" }
/// ```
pub fn extract_keys(lua: &Lua, rec: Table, keys: Table) -> mlua::Result<Value> {
	let out = lua.create_table()?;

	for (idx, key_val) in keys.sequence_values::<Value>().enumerate() {
		let key_val = key_val?;
		let key_str = match key_val {
			Value::String(s) => s,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.extract_keys - Key names must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		};

		let val: Value = rec.get(key_str.clone())?;
		if !val.is_nil() {
			out.set(key_str.clone(), val)?;
			// Remove from original record (in-place)
			rec.set(key_str, Value::Nil)?;
		}
	}

	Ok(Value::Table(out))
}

// region:    --- Tests

#[cfg(test)]
#[path = "shape_keys_tests.rs"]
mod tests;
// endregion: --- Tests
