use std::collections::{BTreeSet, HashSet};

use mlua::{Lua, Table, Value};

use crate::Error;

///
/// Collects string entries from a Lua sequence table and returns them in insertion order.
///
/// ### Example
/// Input table: `{ "id", "name" }`
/// Output vector: `["id", "name"]`
pub fn collect_string_sequence(table: Table, ctx: &str, item_label: &str) -> mlua::Result<Vec<mlua::String>> {
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
pub fn collect_sequence_values(table: Table) -> mlua::Result<Vec<Value>> {
	let mut values: Vec<Value> = Vec::new();

	for value in table.sequence_values::<Value>() {
		values.push(value?);
	}

	Ok(values)
}

/// Collects record tables and returns them alongside the shared string keys.
///
/// Example: [{ id = 1, name = "A" }, { id = 2, name = "B" }] -> keys ["id", "name"].
///
/// Converts a sequence of record tables into a vector of tables plus the shared string keys across all records.
///
/// ### Example
/// Input records: `{ { id = 1, name = "A" }, { id = 2, name = "B" } }`
/// Output: `(rows, {"id", "name"})`
pub fn collect_rows_and_intersection(recs: Table, ctx: &str) -> mlua::Result<(Vec<Table>, BTreeSet<String>)> {
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
