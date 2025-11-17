use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::{Error, Result};
use mlua::{Lua, Table, Value};
use std::collections::{BTreeSet, HashSet};

/// Process correctly the lua eval result
/// (Used by the lua engine eval, and test)
pub fn process_lua_eval_result(_lua: &Lua, res: mlua::Result<Value>, script: &str) -> Result<Value> {
	let res = match res {
		Ok(res) => res,
		Err(err) => return Err(Error::from_error_with_script(&err, script)),
	};

	let res = match res {
		// This is when we d with pcall(...), see test_lua_json_parse_invalid
		Value::Error(err) => {
			return Err(Error::from_error_with_script(&err, script));
			// return Err(Error::from(&*err));
		}
		res => res,
	};

	Ok(res)
}

// region:    --- mlua::Value utils

// Return a Vec<String> from a lua Value which can be String or Array of strings
pub fn into_vec_of_strings(value: Value, err_prefix: &'static str) -> mlua::Result<Vec<String>> {
	match value {
		// If the value is a string, return a Vec with that single string.
		Value::String(lua_string) => {
			let string_value = lua_string.to_str()?.to_string();
			Ok(vec![string_value])
		}

		// If the value is a table, try to interpret it as a list of strings.
		Value::Table(lua_table) => {
			let mut result = Vec::new();

			// Iterate over the table to collect strings.
			for pair in lua_table.sequence_values::<String>() {
				match pair {
					Ok(s) => result.push(s),
					Err(_) => {
						return Err(mlua::Error::FromLuaConversionError {
							from: "table",
							to: "Vec<String>".to_string(),
							message: Some(format!("{err_prefix} - Table contains non-string values")),
						});
					}
				}
			}

			Ok(result)
		}

		// Otherwise, return an error because the value is neither a string nor a list.
		_ => Err(mlua::Error::FromLuaConversionError {
			from: "unknown",
			to: "Vec<String>".to_string(),
			message: Some(format!("{err_prefix} - Expected a string or a list of strings")),
		}),
	}
}

pub fn into_option_string(value: mlua::Value, err_prefix: &str) -> mlua::Result<Option<String>> {
	match value {
		Value::Nil => Ok(None),
		Value::String(string) => Ok(Some(string.to_string_lossy())),
		other => Err(crate::Error::Custom(format!(
			"{err_prefix} - accepted argument types are String or Nil, but was {type_name}",
			type_name = other.type_name()
		))
		.into()),
	}
}

/// Pragmatic way to get a string property from an option lua value
/// TODO: To refactor/clean later
pub fn get_value_prop_as_string(
	value: Option<&mlua::Value>,
	prop_name: &str,
	err_prefix: &str,
) -> mlua::Result<Option<String>> {
	let Some(value) = value else { return Ok(None) };

	let table = value.as_table().ok_or_else(|| {
		crate::Error::custom(format!(
			"{err_prefix} - value should be of type lua table, but was of another type."
		))
	})?;

	match table.get::<Option<Value>>(prop_name)? {
		Some(Value::String(string)) => {
			// TODO: probaby need to normalize_dir to remove the eventual end "/"
			Ok(Some(string.to_string_lossy()))
		}
		Some(_other) => Err(crate::Error::custom(format!(
			"{err_prefix} options.base_dir must be of type string is present"
		))
		.into()),
		None => Ok(None),
	}
}

// endregion: --- mlua::Value utils

// region:    --- Common Paths Support

pub fn path_exits(runtime: &Runtime, path: &str) -> bool {
	let dir_context = runtime.dir_context();
	// Resolve the path relative to the workspace directory
	let full_path = dir_context
		.resolve_path(runtime.session(), path.into(), PathResolver::WksDir, None)
		.ok();

	full_path.map(|p| p.exists()).unwrap_or(false)
}

// endregion: --- Common Paths Support

// region:    --- Lua Shape Support

pub fn expect_table(value: Value, ctx: &str, item_label: &str) -> mlua::Result<Table> {
	match value {
		Value::Table(table) => Ok(table),
		other => Err(Error::custom(format!(
			"{ctx} - {item_label} must be a table. Found '{}'",
			other.type_name()
		))
		.into()),
	}
}

///
/// Collects string entries from a Lua sequence table and returns them in insertion order.
///
/// ### Example
/// Input table: `{ "id", "name" }`
/// Output vector: `["id", "name"]`
pub fn collect_string_sequence(value: Value, ctx: &str, item_label: &str) -> mlua::Result<Vec<mlua::String>> {
	let table = expect_table(value, ctx, item_label)?;
	let mut items: Vec<mlua::String> = Vec::new();

	for (idx, value) in table.sequence_values::<Value>().enumerate() {
		let value = value?;
		match value {
			Value::String(s) => items.push(s),
			other => {
				return Err(Error::custom(format!(
					"{ctx} - {item_label} must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		}
	}

	Ok(items)
}

///
/// Collects arbitrary Lua sequence values into a `Vec<Value>` while preserving their order.
///
/// ### Example
/// Input table: `{ 1, "Alice", true }`
/// Output vector: `[1, "Alice", true]`
pub fn collect_sequence_values(value: Value, ctx: &str, item_label: &str) -> mlua::Result<Vec<Value>> {
	let table = expect_table(value, ctx, item_label)?;
	let mut values: Vec<Value> = Vec::new();

	for value in table.sequence_values::<Value>() {
		values.push(value?);
	}

	Ok(values)
}

///
/// Collects record tables and returns them alongside the shared string keys.
///
/// Example: [{ id = 1, name = "A" }, { id = 2, name = "B" }] -> keys ["id", "name"].
///
/// Converts a sequence of record tables into a vector of tables plus the shared string keys across all records.
///
/// ### Example
/// Input records: `{ { id = 1, name = "A" }, { id = 2, name = "B" } }`
/// Output: `(rows, {"id", "name"})`
pub fn collect_rows_and_intersection(recs: Value, ctx: &str) -> mlua::Result<(Vec<Table>, BTreeSet<String>)> {
	let recs = expect_table(recs, ctx, "Records list")?;
	let mut rows: Vec<Table> = Vec::new();

	for row_val in recs.sequence_values::<Value>() {
		let row_val = row_val?;
		let row_tbl = match row_val {
			Value::Table(t) => t,
			other => {
				return Err(Error::custom(format!(
					"{ctx} - Each record must be a table. Found '{}'",
					other.type_name()
				))
				.into());
			}
		};
		rows.push(row_tbl);
	}

	if rows.is_empty() {
		return Ok((rows, BTreeSet::new()));
	}

	let mut intersect: Option<HashSet<String>> = None;

	for row in &rows {
		let mut keys_this_row: HashSet<String> = HashSet::new();

		for pair in row.pairs::<Value, Value>() {
			let (key, _value) = pair?;
			let key_str = match key {
				Value::String(s) => s.to_string_lossy(),
				other => {
					return Err(Error::custom(format!(
						"{ctx} - Record keys must be strings. Found key of type '{}'",
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

	let ordered_keys = intersect.unwrap_or_default().into_iter().collect::<BTreeSet<_>>();

	Ok((rows, ordered_keys))
}

///
/// Builds a columnar table from the provided rows and ordered keys.
///
/// Example: rows [{ id = 1 }, { id = 2 }] -> { id = { 1, 2 } }.
///
/// Builds a columnar table from row tables using the provided ordered keys.
///
/// ### Example
/// Input rows: `[ { id = 1 }, { id = 2 } ]` with keys `{ "id" }`
/// Output table: `{ id = { 1, 2 } }`
pub fn build_columnar_table(lua: &Lua, rows: &[Table], ordered_keys: &BTreeSet<String>) -> mlua::Result<Table> {
	let out = lua.create_table()?;

	for key in ordered_keys {
		let col = lua.create_table()?;
		for (idx, row) in rows.iter().enumerate() {
			let val: Value = row.get(key.as_str())?;
			col.set(idx + 1, val)?;
		}
		out.set(key.as_str(), col)?;
	}

	Ok(out)
}

// endregion: --- Lua Shape Support
