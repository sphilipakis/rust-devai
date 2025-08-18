//! Defines the `aip.shape` module, used in the Lua engine.
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
//!

use crate::runtime::Runtime;
use crate::{Error, Result};
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let to_record_fn =
		lua.create_function(move |lua, (names, values): (Table, Table)| to_record(lua, names, values))?;
	let to_records_fn = lua.create_function(move |lua, (names, rows): (Table, Table)| to_records(lua, names, rows))?;
	table.set("to_record", to_record_fn)?;
	table.set("to_records", to_records_fn)?;

	Ok(table)
}

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
fn to_record(lua: &Lua, names: Table, values: Table) -> mlua::Result<Value> {
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
fn to_records(lua: &Lua, names: Table, rows: Table) -> mlua::Result<Value> {
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

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
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
}

// endregion: --- Tests
