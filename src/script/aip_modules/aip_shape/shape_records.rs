//! Defines the `aip.shape` helpers used in the Lua engine.
//!
//!
//! ## Lua documentation
//!
//! The `aip.shape` module exposes helpers to shape records built from lists of values (each entry is a heterogeneous list of values) sourced from arrays/lists.
//!
//! `record` means the object (i.e. dictionary) format.
//!
//! ### Functions
//!
//! - `aip.shape.to_record(names: string[], values: any[]) -> object`
//! - `aip.shape.to_records(names: string[], rows: any[][]) -> object[]`
//! - `aip.shape.record_to_values(record: object, names?: string[]): any[]`
//! - `aip.shape.columnar_to_records(cols: { [string]: any[] }): object[]`
//! - `aip.shape.records_to_columnar(recs: object[]): { [string]: any[] }`
//!

use super::support::{
	build_columnar_table, collect_rows_and_intersection, collect_sequence_values, collect_string_sequence,
};
use crate::Error;
use crate::script::NullSentinel;
use mlua::{Lua, Table, Value};

/// ## Lua Documentation
///
/// Build a single record (i.e., object) from a list of column names and a list of values.
///
/// ```lua
/// -- API Signature
/// aip.shape.to_record(names: string[], values: any[]): object
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
/// - `object` - A Lua table with keys from `names` and values from `values`.
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
	let name_vec = collect_string_sequence(names, "aip.shape.to_record", "Column names")?;

	// Collect values as arbitrary Lua values
	let vals_vec = collect_sequence_values(values)?;

	let limit = core::cmp::min(name_vec.len(), vals_vec.len());

	let rec = lua.create_table()?;
	for i in 0..limit {
		// NOTE: Should always be fine, but avoid [.] by best practice
		if let (Some(name), Some(val)) = (name_vec.get(i), vals_vec.get(i)) {
			rec.set(name.clone(), val.clone())?;
		}
	}

	Ok(Value::Table(rec))
}

/// ## Lua Documentation
///
/// Build multiple records (i.e., objects) from a list of column names and a list of value lists (each entry is a heterogeneous list of values).
///
/// ```lua
/// -- API Signature
/// aip.shape.to_records(names: string[], value_lists: any[][]): object[]
/// ```
///
/// - Truncates each list of values to the shorter length between `names` and the provided values.
/// - Extra names without corresponding values are ignored.
/// - Extra values inside the same list without corresponding names are ignored.
///
/// ### Errors
///
/// - If `names` contains a non-string entry, an error is returned.
/// - If any list of values is not a table (list), an error is returned.
///
/// ### Example
///
/// ```lua
/// local names = { "id", "name" }
/// local value_lists = {
///   { 1, "Alice" },
///   { 2, "Bob"   },
/// }
/// local out = aip.shape.to_records(names, value_lists)
/// -- out == {
/// --   { id = 1, name = "Alice" },
/// --   { id = 2, name = "Bob"   },
/// -- }
/// ```
pub fn to_records(lua: &Lua, names: Table, value_lists: Table) -> mlua::Result<Value> {
	// Validate and collect column names as strings
	let name_vec = collect_string_sequence(names, "aip.shape.to_records", "Column names")?;

	// Build records
	let out = lua.create_table()?;
	let mut out_idx = 1usize;

	for row_val in value_lists.sequence_values::<Value>() {
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
		let vals_vec = collect_sequence_values(row_tbl)?;

		let limit = core::cmp::min(name_vec.len(), vals_vec.len());
		let rec = lua.create_table()?;
		for i in 0..limit {
			if let (Some(name), Some(val)) = (name_vec.get(i), vals_vec.get(i)) {
				rec.set(name.clone(), val.clone())?;
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
/// aip.shape.record_to_values(record: object, names?: string[]): any[]
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
			let ordered_names = collect_string_sequence(names, "aip.shape.record_to_values", "Column names")?;

			for name_str in ordered_names {
				let val: Value = rec.get(name_str.clone())?;
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
/// Convert a column-oriented table into a list of records produced from lists of values.
///
/// ```lua
/// -- API Signature
/// aip.shape.columnar_to_records(cols: { [string]: any[] }): object[]
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
		let vec_vals = collect_sequence_values(tbl)?;
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
/// Convert a list of record objects into a column-oriented table.
/// Uses the intersection of string keys present across all records to ensure rectangular output.
///
/// ```lua
/// -- API Signature
/// aip.shape.records_to_columnar(recs: object[]): { [string]: any[] }
/// ```
///
/// - Each record must be an object (Lua table).
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
	// Collect rows as tables, validating each entry
	let (rows, ordered_keys) = collect_rows_and_intersection(recs, "aip.shape.records_to_columnar")?;

	// Early return: no rows -> empty columns table
	if rows.is_empty() {
		return Ok(Value::Table(lua.create_table()?));
	}

	// Compute the intersection of string keys across all records
	// Deterministic order for output columns
	if ordered_keys.is_empty() {
		return Ok(Value::Table(lua.create_table()?));
	}

	// Build columns
	let out = build_columnar_table(lua, &rows, &ordered_keys)?;

	Ok(Value::Table(out))
}

// region:    --- Tests

#[cfg(test)]
#[path = "shape_records_tests.rs"]
mod tests;

// endregion: --- Tests
