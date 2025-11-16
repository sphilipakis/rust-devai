//! Defines the `aip.shape` helpers used in the Lua engine.
//!
//!
//! ## Lua documentation
//!
//! The `aip.shape` module exposes helpers to shape records (row objects) from arrays/lists.
//!
//! ### Functions
//!
//! - `aip.shape.to_record(names: string[], values: any[]) -> table`
//! - `aip.shape.to_records(names: string[], rows: any[][]) -> table[]`
//! - `aip.shape.record_to_values(record: table, names?: string[]): any[]`
//! - `aip.shape.columnar_to_records(cols: { [string]: any[] }): table[]`
//! - `aip.shape.records_to_columnar(recs: table[]): { [string]: any[] }`
//!

use crate::Error;
use mlua::{Lua, Table, Value};

use crate::script::lua_null::NullSentinel;

/// ## Lua Documentation
///
/// Build a single record (row object) from a list of column names and a list of values.
///
/// ```lua
/// -- API Signature
/// aip.shape.to_record(names: string[], values: any[]): table
/// ```
///
/// ### Example:
///
/// ```lua
/// local names  = { "id", "name", "email" }
/// local values = { 1, "Alice", "alice@example.com" }
/// local rec = aip.shape.to_record(names, values)
/// -- rec == { id = 1, name = "Alice", email = "alice@example.com" }
/// ```
///
/// ### Arguments
///
/// - `names: string[]`  - Array of column names (Lua list).
/// - `values: any[]`    - Array of values (Lua list).
///
/// ### Returns
///
/// - `table` - A Lua table with keys from `names` and values from `values`.
///
/// ### Errors
///
/// - If `names` contains a non-string entry, an error is returned.
///
/// ### Notes:
///
/// - Truncates to the shorter length of the two lists.
/// - If a name is not a string, an error is returned.
/// - This function is lenient and truncates to the shorter length between `names` and `values`.
/// - Extra names without corresponding values are ignored.
/// - Extra values without corresponding names are ignored.
///
pub fn to_record(lua: &Lua, names: Table, values: Table) -> mlua::Result<Value> {
	// NOTE: Here we keep the data in the Lua space as there is no need to make them cross boundaries.

	// Collect names as strings with validation
	let mut name_vec: Vec<mlua::String> = Vec::new();
	for (idx, v) in names.sequence_values::<Value>().enumerate() {
		let v = v?;
		match v {
			Value::String(s) => name_vec.push(s),
			other => {
				return Err(Error::custom(format!(
					"aip.shape.to_record - Column names must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		}
	}

	// Collect values as arbitrary Lua values
	let mut vals_vec: Vec<Value> = Vec::new();
	for v in values.sequence_values::<Value>() {
		vals_vec.push(v?);
	}

	let limit = core::cmp::min(name_vec.len(), vals_vec.len());

	let rec = lua.create_table()?;
	for i in 0..limit {
		// NOTE: Should always be fine, but avoid [.] by best practice
		if let (Some(name), Some(val)) = (name_vec.get(i), vals_vec.get(i)) {
			rec.set(name, val)?;
		}
	}

	Ok(Value::Table(rec))
}

/// ## Lua Documentation
///
/// Build multiple records (row objects) from a list of column names and a list of rows.
///
/// ```lua
/// -- API Signature
/// aip.shape.to_records(names: string[], rows: any[][]): table[]
/// ```
///
/// - Truncates each row to the shorter length between `names` and the row values.
/// - Extra names without corresponding values are ignored.
/// - Extra row values without corresponding names are ignored.
///
/// ### Errors
///
/// - If `names` contains a non-string entry, an error is returned.
/// - If any row is not a table (list), an error is returned.
///
/// ### Example
///
/// ```lua
/// local names = { "id", "name" }
/// local rows  = {
///   { 1, "Alice" },
///   { 2, "Bob"   },
/// }
/// local out = aip.shape.to_records(names, rows)
/// -- out == {
/// --   { id = 1, name = "Alice" },
/// --   { id = 2, name = "Bob"   },
/// -- }
/// ```
pub fn to_records(lua: &Lua, names: Table, rows: Table) -> mlua::Result<Value> {
	// Validate and collect column names as strings
	let mut name_vec: Vec<mlua::String> = Vec::new();
	for (idx, v) in names.sequence_values::<Value>().enumerate() {
		let v = v?;
		match v {
			Value::String(s) => name_vec.push(s),
			other => {
				return Err(Error::custom(format!(
					"aip.shape.to_records - Column names must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		}
	}

	// Build records
	let out = lua.create_table()?;
	let mut out_idx = 1usize;

	for row_val in rows.sequence_values::<Value>() {
		let row_val = row_val?;
		let row_tbl = match row_val {
			Value::Table(t) => t,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.to_records - Each row must be a table (list). Found '{}'",
					other.type_name()
				))
				.into());
			}
		};

		// Collect row values
		let mut vals_vec: Vec<Value> = Vec::new();
		for v in row_tbl.sequence_values::<Value>() {
			vals_vec.push(v?);
		}

		let limit = core::cmp::min(name_vec.len(), vals_vec.len());
		let rec = lua.create_table()?;
		for i in 0..limit {
			if let (Some(name), Some(val)) = (name_vec.get(i), vals_vec.get(i)) {
				rec.set(name, val)?;
			}
		}

		out.set(out_idx, rec)?;
		out_idx += 1;
	}

	Ok(Value::Table(out))
}

/// ## Lua Documentation
///
/// Convert a single record into an array (Lua list) of values.
///
/// ```lua
/// -- API Signature
/// aip.shape.record_to_values(record: table, names?: string[]): any[]
/// ```
///
/// - When `names` is provided, values are returned in the order of `names`.
///   - Missing keys yield `null` sentinel entries in the result list.
///   - If `names` contains a non-string entry, an error is returned.
/// - When `names` is not provided, values are returned in alphabetical order of the record's string keys.
///   - Non-string keys are ignored.
///
/// ### Examples
///
/// ```lua
/// local rec = { id = 1, name = "Alice", email = "a@x.com" }
/// local v1  = aip.shape.record_to_values(rec)                
/// -- { 1, "a@x.com", "Alice" } (alpha by keys: email, id, name)
///
/// local v2  = aip.shape.record_to_values(rec, { "name", "id", "missing" })
/// -- { "Alice", 1, null }
/// ```
pub fn record_to_values(lua: &Lua, rec: Table, names_opt: Option<Table>) -> mlua::Result<Value> {
	let out = lua.create_table()?;

	match names_opt {
		Some(names) => {
			let mut idx = 1usize;

			for (i, name_val) in names.sequence_values::<Value>().enumerate() {
				let name_val = name_val?;
				let name_str = match name_val {
					Value::String(s) => s,
					other => {
						return Err(Error::custom(format!(
							"aip.shape.record_to_values - Column names must be strings. Found '{}' at index {}",
							other.type_name(),
							i + 1
						))
						.into());
					}
				};

				let val: Value = rec.get(name_str)?;
				if let Value::Nil = val {
					let na = lua.create_userdata(NullSentinel)?;
					out.set(idx, na)?;
				} else {
					out.set(idx, val)?;
				}
				idx += 1;
			}
		}
		None => {
			// Collect string keys and sort them
			let mut keys: Vec<String> = Vec::new();
			for pair in rec.pairs::<Value, Value>() {
				let (k, _v) = pair?;
				if let Value::String(s) = k {
					keys.push(s.to_string_lossy());
				}
			}
			keys.sort();

			for (i, k) in keys.iter().enumerate() {
				let v: Value = rec.get(k.as_str())?;
				out.set(i + 1, v)?;
			}
		}
	}

	Ok(Value::Table(out))
}

/// ## Lua Documentation
///
/// Convert a column-oriented table into a list of row records.
///
/// ```lua
/// -- API Signature
/// aip.shape.columnar_to_records(cols: { [string]: any[] }): table[]
/// ```
///
/// - All keys in `cols` must be strings (column names).
/// - Each column value must be a table (Lua list).
/// - All columns must have the same length; otherwise an error is returned.
///
/// ### Example:
///
/// ```lua
/// local cols = {
///   id    = { 1, 2, 3 },
///   name  = { "Alice", "Bob", "Cara" },
///   email = { "a@x.com", "b@x.com", "c@x.com" },
/// }
/// local recs = aip.shape.columnar_to_records(cols)
/// -- recs == {
/// --   { id = 1, name = "Alice", email = "a@x.com" },
/// --   { id = 2, name = "Bob",   email = "b@x.com" },
/// --   { id = 3, name = "Cara",  email = "c@x.com" },
/// -- }
/// ```
pub fn columnar_to_records(lua: &Lua, cols: Table) -> mlua::Result<Value> {
	// Collect column names and their values (as vectors)
	let mut col_names: Vec<mlua::String> = Vec::new();
	let mut col_values: Vec<Vec<Value>> = Vec::new();
	let mut expected_len: Option<usize> = None;

	for pair in cols.pairs::<Value, Value>() {
		let (key, val) = pair?;

		// Keys must be strings
		let key_str = match key {
			Value::String(s) => s,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.columnar_to_records - Column keys must be strings. Found '{}'",
					other.type_name()
				))
				.into());
			}
		};

		// Values must be tables (lists)
		let tbl = match val {
			Value::Table(t) => t,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.columnar_to_records - Each column must be a table (list). Column '{}' was '{}'",
					key_str.to_string_lossy(),
					other.type_name()
				))
				.into());
			}
		};

		// Collect sequence values
		let mut vec_vals: Vec<Value> = Vec::new();
		for v in tbl.sequence_values::<Value>() {
			vec_vals.push(v?);
		}
		let len = vec_vals.len();

		// Length consistency check
		if let Some(exp) = expected_len {
			if len != exp {
				return Err(Error::custom(format!(
					"aip.shape.columnar_to_records - All columns must have the same length. Column '{}' has length {}, expected {}",
					key_str.to_string_lossy(),
					len,
					exp
				))
				.into());
			}
		} else {
			expected_len = Some(len);
		}

		col_names.push(key_str);
		col_values.push(vec_vals);
	}

	let row_count = expected_len.unwrap_or(0);
	let out = lua.create_table()?;

	// Build each record
	for i in 0..row_count {
		let rec = lua.create_table()?;
		for (idx, name) in col_names.iter().enumerate() {
			if let Some(vals) = col_values.get(idx)
				&& let Some(val) = vals.get(i)
			{
				rec.set(name, val.clone())?;
			}
		}
		out.set(i + 1, rec)?;
	}

	Ok(Value::Table(out))
}

/// ## Lua Documentation
///
/// Convert a list of record tables into a column-oriented table.
/// Uses the intersection of string keys present across all records to ensure rectangular output.
///
/// ```lua
/// -- API Signature
/// aip.shape.records_to_columnar(recs: table[]): { [string]: any[] }
/// ```
///
/// - Each record must be a table.
/// - All keys must be strings; if any non-string key is found, an error is returned.
/// - The output contains only the keys present in every record (set intersection).
/// ### Example
///
/// ```lua
/// local recs = {
///   { id = 1, name = "Alice" },
///   { id = 2, name = "Bob"   },
/// }
/// local cols = aip.shape.records_to_columnar(recs)
/// -- cols == {
/// --   id   = { 1, 2 },
/// --   name = { "Alice", "Bob" },
/// -- }
/// ```
pub fn records_to_columnar(lua: &Lua, recs: Table) -> mlua::Result<Value> {
	use std::collections::{BTreeSet, HashSet};

	// Collect rows as tables, validating each entry
	let mut rows: Vec<Table> = Vec::new();
	for row_val in recs.sequence_values::<Value>() {
		let row_val = row_val?;
		let row_tbl = match row_val {
			Value::Table(t) => t,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.records_to_columnar - Each record must be a table. Found '{}'",
					other.type_name()
				))
				.into());
			}
		};
		rows.push(row_tbl);
	}

	// Early return: no rows -> empty columns table
	if rows.is_empty() {
		return Ok(Value::Table(lua.create_table()?));
	}

	// Compute the intersection of string keys across all records
	let mut intersect: Option<HashSet<String>> = None;

	for row in &rows {
		let mut keys_this_row: HashSet<String> = HashSet::new();

		for pair in row.pairs::<Value, Value>() {
			let (k, _v) = pair?;

			let key_str = match k {
				Value::String(s) => s.to_string_lossy(),
				other => {
					return Err(Error::custom(format!(
						"aip.shape.records_to_columnar - Record keys must be strings. Found key of type '{}'",
						other.type_name()
					))
					.into());
				}
			};
			keys_this_row.insert(key_str);
		}

		intersect = Some(match intersect.take() {
			None => keys_this_row,
			Some(prev) => prev.intersection(&keys_this_row).cloned().collect(),
		});
	}

	let keys = intersect.unwrap_or_default();
	// Deterministic order for output columns
	let mut ordered_keys: BTreeSet<String> = BTreeSet::new();
	for k in keys {
		ordered_keys.insert(k);
	}

	// Build columns
	let out = lua.create_table()?;
	for key in ordered_keys {
		let col = lua.create_table()?;
		for (idx, row) in rows.iter().enumerate() {
			let val: Value = row.get(key.as_str())?;
			col.set(idx + 1, val)?;
		}
		out.set(key.as_str(), col)?;
	}

	Ok(Value::Table(out))
}

// region:    --- Tests

#[cfg(test)]
#[path = "shape_records_tests.rs"]
mod tests;

// endregion: --- Tests
