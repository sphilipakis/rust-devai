//! Defines the `aip.shape` helpers used in the Lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.shape` module exposes helpers to shape records (row objects) from arrays/lists.
//!
//! ### Functions
//!
//! - `aip.shape.to_record(names: string[], values: any[]) -> table`
//! - `aip.shape.to_records(names: string[], rows: any[][]) -> table[]`
//! - `aip.shape.columns_to_records(cols: { [string]: any[] }): table[]`
//!

use crate::Error;
use mlua::{Lua, Table, Value};

/// ## Lua Documentation
/// ---
/// Build a single record (row object) from a list of column names and a list of values.
///
/// ```lua
/// -- API Signature
/// aip.shape.to_record(names: string[], values: any[]): table
/// ```
///
/// ### Example:
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
/// ---
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
/// ---
/// Convert a column-oriented table into a list of row records.
///
/// ```lua
/// -- API Signature
/// aip.shape.columns_to_records(cols: { [string]: any[] }): table[]
/// ```
///
/// - All keys in `cols` must be strings (column names).
/// - Each column value must be a table (Lua list).
/// - All columns must have the same length; otherwise an error is returned.
///
/// ### Example:
/// ```lua
/// local cols = {
///   id    = { 1, 2, 3 },
///   name  = { "Alice", "Bob", "Cara" },
///   email = { "a@x.com", "b@x.com", "c@x.com" },
/// }
/// local recs = aip.shape.columns_to_records(cols)
/// -- recs == {
/// --   { id = 1, name = "Alice", email = "a@x.com" },
/// --   { id = 2, name = "Bob",   email = "b@x.com" },
/// --   { id = 3, name = "Cara",  email = "c@x.com" },
/// -- }
/// ```
pub fn columns_to_records(lua: &Lua, cols: Table) -> mlua::Result<Value> {
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
					"aip.shape.columns_to_records - Column keys must be strings. Found '{}'",
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
					"aip.shape.columns_to_records - Each column must be a table (list). Column '{}' was '{}'",
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
					"aip.shape.columns_to_records - All columns must have the same length. Column '{}' has length {}, expected {}",
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
/// ---
/// Convert a list of record tables into a column-oriented table.
/// Uses the intersection of string keys present across all records to ensure rectangular output.
///
/// ```lua
/// -- API Signature
/// aip.shape.records_to_columns(recs: table[]): { [string]: any[] }
/// ```
///
/// - Each record must be a table.
/// - All keys must be strings; if any non-string key is found, an error is returned.
/// - The output contains only the keys present in every record (set intersection).
pub fn records_to_columns(lua: &Lua, recs: Table) -> mlua::Result<Value> {
	use std::collections::{BTreeSet, HashSet};

	// Collect rows as tables, validating each entry
	let mut rows: Vec<Table> = Vec::new();
	for row_val in recs.sequence_values::<Value>() {
		let row_val = row_val?;
		let row_tbl = match row_val {
			Value::Table(t) => t,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.records_to_columns - Each record must be a table. Found '{}'",
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
						"aip.shape.records_to_columns - Record keys must be strings. Found key of type '{}'",
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
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules::aip_shape::init_module;
	use serde_json::json;

	#[tokio::test]
	async fn test_lua_aip_shape_to_record_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local names  = { "id", "name", "email" }
            local values = { 1, "Alice", "alice@example.com" }
            return aip.shape.to_record(names, values)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!({
			"id": 1,
			"name": "Alice",
			"email": "alice@example.com"
		});
		assert_eq!(res, expected);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_to_record_extra_values_truncated() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local names  = { "id", "name" }
            local values = { 1, "Alice", "EXTRA" }
            return aip.shape.to_record(names, values)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!({
			"id": 1,
			"name": "Alice"
		});
		assert_eq!(res, expected);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_to_record_extra_names_truncated() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local names  = { "id", "name", "email" }
            local values = { 2, "Bob" }
            return aip.shape.to_record(names, values)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!({
			"id": 2,
			"name": "Bob"
		});
		assert_eq!(res, expected);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_to_record_invalid_name_type() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local ok, err = pcall(function()
                return aip.shape.to_record({ "id", 123, "email" }, { 3, "Cara", "c@x.com" })
            end)
            if ok then
                return "should not reach"
            else
                return err
            end
        "#;

		// -- Exec
		let res = eval_lua(&lua, script);

		// -- Check
		let Err(err) = res else {
			panic!("Expected error, got {res:?}");
		};
		let err_str = err.to_string();
		assert_contains(&err_str, "aip.shape.to_record - Column names must be strings");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_to_records_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local names = { "id", "name" }
            local rows  = {
              { 1, "Alice" },
              { 2, "Bob"   },
            }
            return aip.shape.to_records(names, rows)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!([
			{ "id": 1, "name": "Alice" },
			{ "id": 2, "name": "Bob" }
		]);
		assert_eq!(res, expected);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_to_records_rows_var_len_truncation() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local names = { "id", "name", "email" }
            local rows  = {
              { 1, "Alice" },                    -- shorter row
              { 2, "Bob", "b@x.com", "EXTRA" },  -- longer row
            }
            return aip.shape.to_records(names, rows)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!([
			{ "id": 1, "name": "Alice" },
			{ "id": 2, "name": "Bob", "email": "b@x.com" }
		]);
		assert_eq!(res, expected);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_to_records_row_not_table_err() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local names = { "id", "name" }
            local rows  = {
              { 1, "Alice" },
              "INVALID_ROW"
            }
            local ok, err = pcall(function()
              return aip.shape.to_records(names, rows)
            end)
            if ok then
              return "should not reach"
            else
              return err
            end
        "#;

		// -- Exec
		let res = eval_lua(&lua, script);

		// -- Check
		let Err(err) = res else {
			panic!("Expected error, got {res:?}");
		};
		let err_str = err.to_string();
		assert_contains(&err_str, "aip.shape.to_records - Each row must be a table (list)");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_to_records_invalid_name_type() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local names = { "id", 999, "email" }
            local rows  = { { 1, "Alice", "a@x.com" } }
            local ok, err = pcall(function()
              return aip.shape.to_records(names, rows)
            end)
            if ok then
              return "should not reach"
            else
              return err
            end
        "#;

		// -- Exec
		let res = eval_lua(&lua, script);

		// -- Check
		let Err(err) = res else {
			panic!("Expected error, got {res:?}");
		};
		let err_str = err.to_string();
		assert_contains(&err_str, "aip.shape.to_records - Column names must be strings");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_columns_to_records_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local cols = {
              id    = { 1, 2, 3 },
              name  = { "Alice", "Bob", "Cara" },
              email = { "a@x.com", "b@x.com", "c@x.com" },
            }
            return aip.shape.columns_to_records(cols)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!([
			{ "id": 1, "name": "Alice", "email": "a@x.com" },
			{ "id": 2, "name": "Bob",   "email": "b@x.com" },
			{ "id": 3, "name": "Cara",  "email": "c@x.com" }
		]);
		assert_eq!(res, expected);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_columns_to_records_len_mismatch_err() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
            local cols = {
              id   = { 1, 2 },
              name = { "Alice" }, -- mismatch length
            }
            local ok, err = pcall(function()
              return aip.shape.columns_to_records(cols)
            end)
            if ok then
              return "should not reach"
            else
              return err
            end
        "#;

		// -- Exec
		let res = eval_lua(&lua, script);

		// -- Check
		let Err(err) = res else {
			panic!("Expected error, got {res:?}");
		};
		let err_str = err.to_string();
		assert_contains(
			&err_str,
			"aip.shape.columns_to_records - All columns must have the same length",
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_records_to_columns_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
			local recs = {
			  { id = 1, name = "Alice" },
			  { id = 2, name = "Bob" },
			}
			return aip.shape.records_to_columns(recs)
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!({
			"id":   [1, 2],
			"name": ["Alice", "Bob"]
		});
		assert_eq!(res, expected);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_records_to_columns_intersection() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
			local recs = {
			  { id = 1, name = "Alice", email = "a@x.com" },
			  { id = 2, name = "Bob" }, -- missing email
			}
			return aip.shape.records_to_columns(recs)
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let expected = json!({
			"id":   [1, 2],
			"name": ["Alice", "Bob"]
			// 'email' omitted due to intersection
		});
		assert_eq!(res, expected);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_records_to_columns_row_not_table_err() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
			local recs = {
			  { id = 1, name = "Alice" },
			  "INVALID_ROW"
			}
			local ok, err = pcall(function()
			  return aip.shape.records_to_columns(recs)
			end)
			if ok then
			  return "should not reach"
			else
			  return err
			end
		"#;

		// -- Exec
		let res = eval_lua(&lua, script);

		// -- Check
		let Err(err) = res else {
			panic!("Expected error, got {res:?}");
		};
		let err_str = err.to_string();
		assert_contains(&err_str, "aip.shape.records_to_columns - Each record must be a table");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_shape_records_to_columns_non_string_key_err() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "shape").await?;
		let script = r#"
			local function make_bad()
			  local t = { id = 1, name = "Alice" }
			  t[123] = "bad" -- non-string key
			  return t
			end
			local recs = { make_bad() }
			local ok, err = pcall(function()
			  return aip.shape.records_to_columns(recs)
			end)
			if ok then
			  return "should not reach"
			else
			  return err
			end
		"#;

		// -- Exec
		let res = eval_lua(&lua, script);

		// -- Check
		let Err(err) = res else {
			panic!("Expected error, got {res:?}");
		};
		let err_str = err.to_string();
		assert_contains(&err_str, "aip.shape.records_to_columns - Record keys must be strings");

		Ok(())
	}








}

// endregion: --- Tests
