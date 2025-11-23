//! Defines the `aip.csv` module, used in the lua engine.
//!
//!
//! ## Lua documentation
//!
//! The `aip.csv` module exposes helpers to parse CSV content or a single CSV row, with
//! customizable delimiter, quote, escape, trimming, header handling, and comment skipping.
//!
//! Options are shared for both functions; fields not applicable to `parse_row` are ignored.
//!
//! ### Functions
//!
//! - `aip.csv.parse_row(row: string, options?: CsvOptions): string[]`
//! - `aip.csv.parse(content: string, options?: CsvOptions): CsvContent`
//! - `aip.csv.values_to_row(values: any[]): string`
//! - `aip.csv.value_lists_to_rows(value_lists: any[][]): string[]`
//!
//! ### Related Types
//!
//! Where `CsvOptions` is:
//! ```lua
//! {
//!   delimiter?: string,         -- default ","
//!   quote?: string,             -- default "\""
//!   escape?: string,            -- default "\""
//!   trim_fields?: boolean,      -- default false
//!   has_header?: boolean,       -- default true for parse()
//!   skip_empty_lines?: boolean, -- default true
//!   comment?: string            -- e.g., "#", optional
//! }
//! ```
//!
//! Notes:
//! - `parse_row` ignores: `has_header`, `skip_empty_lines`, and `comment`.
//! - When an option expecting a character is given a multi-character string, only the first byte is used.

use crate::Result;
use crate::runtime::Runtime;
use crate::script::lua_helpers::lua_value_to_serde_value;
use crate::support::W;
use crate::types::CsvOptions;
use mlua::{FromLua as _, IntoLua as _, Lua, Table, Value};

/// Convert a Lua value to a string suitable for CSV.
/// - Strings are returned as is.
/// - Numbers/Booleans are to_string().
/// - Nil/NullSentinel becomes "".
/// - Tables are serialized to JSON.
pub fn lua_value_to_csv_string(value: Value) -> mlua::Result<String> {
	match value {
		Value::String(s) => Ok(s.to_str()?.to_string()),
		Value::Integer(i) => Ok(i.to_string()),
		Value::Number(n) => Ok(n.to_string()),
		Value::Boolean(b) => Ok(b.to_string()), // "true" or "false"
		Value::Nil => Ok("".to_string()),
		Value::UserData(ud) if ud.is::<crate::script::NullSentinel>() => Ok("".to_string()),
		Value::Table(t) => {
			let serde_val = lua_value_to_serde_value(Value::Table(t)).map_err(mlua::Error::external)?;
			serde_json::to_string(&serde_val).map_err(mlua::Error::external)
		}
		other => Err(mlua::Error::external(format!(
			"unsupported value type '{}'",
			other.type_name()
		))),
	}
}

/// Convert a Lua table (list of lists) into a Vec<Vec<String>>.
///
/// Each inner value is converted to a CSV string using `lua_value_to_csv_string`.
pub fn lua_matrix_to_rows(matrix: Table) -> mlua::Result<Vec<Vec<String>>> {
	let mut rows = Vec::new();
	for row_val in matrix.sequence_values::<Value>() {
		let row_val = row_val?;
		if let Value::Table(row_tbl) = row_val {
			let mut row = Vec::new();
			for cell_val in row_tbl.sequence_values::<Value>() {
				row.push(lua_value_to_csv_string(cell_val?)?);
			}
			rows.push(row);
		}
	}
	Ok(rows)
}

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let parse_row_fn =
		lua.create_function(|lua, (row, opts): (String, Option<Value>)| lua_parse_row(lua, row, opts))?;
	let parse_fn =
		lua.create_function(|lua, (content, opts): (String, Option<Value>)| lua_parse(lua, content, opts))?;
	let values_to_row_fn =
		lua.create_function(|lua, (values, opts): (Value, Option<Value>)| lua_values_to_row(lua, values, opts))?;
	let value_lists_to_rows_fn =
		lua.create_function(|lua, (lists, opts): (Value, Option<Value>)| lua_value_lists_to_rows(lua, lists, opts))?;

	table.set("parse_row", parse_row_fn)?;
	table.set("parse", parse_fn)?;
	table.set("values_to_row", values_to_row_fn)?;
	table.set("value_lists_to_rows", value_lists_to_rows_fn)?;

	Ok(table)
}

// region:    --- Lua Fns

/// ## Lua Documentation
///
/// Parse a single CSV row according to the options (delimiter, quote, escape, trim_fields).
/// Non-applicable options (`has_header`, `skip_empty_lines`, `comment`) are ignored.
///
/// ```lua
/// -- API Signature
/// aip.csv.parse_row(row: string, options?: CsvOptions): string[]
/// ```
fn lua_parse_row(lua: &Lua, row: String, opts_val: Option<Value>) -> mlua::Result<Table> {
	let opts = match opts_val {
		Some(v) => CsvOptions::from_lua(v, lua)?,
		None => CsvOptions::default(),
	};

	let row_vec = crate::support::csvs::parse_csv_row(&row, Some(opts))?;

	let table = W(row_vec)
		.into_lua(lua)
		.map_err(|e| mlua::Error::external(format!("Failed to convert row to lua table: {e}")))?;

	match table {
		Value::Table(t) => Ok(t),
		_ => Err(mlua::Error::external("Expected a table")),
	}
}

/// ## Lua Documentation
///
/// Parse CSV content, optionally with header detection and comment skipping.
/// Returns a `CsvContent` table (`{ _type = "CsvContent", headers = string[], rows = string[][] }`).
/// When `has_header` is `false`, the headers array is returned empty rather than `nil`.
/// By default this API treats the first row as headers (`has_header = true`) and skips empty lines
/// (`skip_empty_lines = true`) unless these options are overridden.
///
/// ```lua
/// -- API Signature
/// aip.csv.parse(content: string, options?: CsvOptions): CsvContent
/// ```
/// The returned table matches the `CsvContent` structure (same as `aip.file.load_csv`),
/// including the `_type = "CsvContent"` marker and using an empty `headers` array when `has_header` is false.
fn lua_parse(lua: &Lua, content: String, opts_val: Option<Value>) -> mlua::Result<Value> {
	let opts = match opts_val {
		Some(v) => CsvOptions::from_lua(v, lua)?,
		None => CsvOptions::default(),
	};

	let csv_content = crate::support::csvs::parse_csv(&content, Some(opts))?;

	csv_content.into_lua(lua)
}

/// ## Lua Documentation
///
/// Converts a list of values into a CSV row string.
///
/// ```lua
/// -- API Signature
/// aip.csv.values_to_row(values: any[], options?: CsvOptions): string
/// ```
///
/// - `values`: A list of values (strings, numbers, booleans, nil, or tables).
///   - Tables are converted to JSON strings.
///   - Nil (and null sentinel) are converted to empty strings.
/// - `options`: Optional `CsvOptions` (e.g., `delimiter`, `quote`, `escape`).
fn lua_values_to_row(lua: &Lua, values: Value, opts_val: Option<Value>) -> mlua::Result<String> {
	let opts = match opts_val {
		Some(v) => Some(CsvOptions::from_lua(v, lua)?),
		None => None,
	};
	values_to_row_inner(values, opts, "aip.csv.values_to_row")
}

/// ## Lua Documentation
///
/// Converts a list of list of values into a list of CSV row strings.
///
/// ```lua
/// -- API Signature
/// aip.csv.value_lists_to_rows(value_lists: any[][], options?: CsvOptions): string[]
/// ```
///
/// - `value_lists`: A list of lists of values.
/// - `options`: Optional `CsvOptions` (e.g., `delimiter`, `quote`, `escape`).
fn lua_value_lists_to_rows(lua: &Lua, value_lists: Value, opts_val: Option<Value>) -> mlua::Result<Vec<String>> {
	let table = match value_lists {
		Value::Table(t) => t,
		_ => {
			return Err(mlua::Error::external(
				"aip.csv.value_lists_to_rows - value_lists must be a table (list of lists)",
			));
		}
	};

	let opts = match opts_val {
		Some(v) => Some(CsvOptions::from_lua(v, lua)?),
		None => None,
	};

	let mut rows = Vec::new();
	for (idx, item) in table.sequence_values::<Value>().enumerate() {
		let item = item?;
		let row = values_to_row_inner(item, opts.clone(), "aip.csv.value_lists_to_rows")
			.map_err(|e| mlua::Error::external(format!("Row {}: {}", idx + 1, e)))?;
		rows.push(row);
	}
	Ok(rows)
}

fn values_to_row_inner(values: Value, opts: Option<CsvOptions>, ctx: &str) -> mlua::Result<String> {
	let table = match values {
		Value::Table(t) => t,
		_ => {
			return Err(mlua::Error::external(format!("{ctx} - values must be a table (list)")));
		}
	};

	let mut row_values = Vec::new();

	// Iterate over sequence values (1..N)
	for value in table.sequence_values::<Value>() {
		let value = value?;
		let s = lua_value_to_csv_string(value).map_err(|e| mlua::Error::external(format!("{ctx} - {e}")))?;
		row_values.push(s);
	}

	let csv_row = crate::support::csvs::values_to_csv_row(&row_values, opts).map_err(mlua::Error::external)?;
	Ok(csv_row)
}

// endregion: --- Lua Fns

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_csv;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_aip_csv_parse_row_simple() -> Result<()> {
		let lua = setup_lua(aip_csv::init_module, "csv").await?;
		let res = eval_lua(
			&lua,
			r#"
                local row = 'a,"b,c",d'
                return aip.csv.parse_row(row)
            "#,
		)?;
		assert_eq!(res.x_get_str("/0")?, "a");
		assert_eq!(res.x_get_str("/1")?, "b,c");
		assert_eq!(res.x_get_str("/2")?, "d");
		Ok(())
	}

	#[tokio::test]
	async fn test_aip_csv_parse_with_header_and_comments() -> Result<()> {
		let lua = setup_lua(aip_csv::init_module, "csv").await?;
		let script = r##"
            local content = [[
# comment
name,age
John,30

Jane,25
]]
            local res = aip.csv.parse(content, { has_header = true, comment = "#", skip_empty_lines = true })
            return res
        "##;
		let res = eval_lua(&lua, script)?;
		assert_eq!(res.x_get_str("/headers/0")?, "name");
		assert_eq!(res.x_get_str("/headers/1")?, "age");

		assert_eq!(res.x_get_str("/rows/0/0")?, "John");
		assert_eq!(res.x_get_str("/rows/0/1")?, "30");
		assert_eq!(res.x_get_str("/rows/1/0")?, "Jane");
		assert_eq!(res.x_get_str("/rows/1/1")?, "25");

		Ok(())
	}

	#[tokio::test]
	async fn test_aip_csv_values_to_row() -> Result<()> {
		let lua = setup_lua(aip_csv::init_module, "csv").await?;

		// Test simple values
		let res = eval_lua(&lua, r#"return aip.csv.values_to_row({"a", 123, true, nil})"#)?;
		let s = res.as_str().ok_or("Should be string")?;
		assert_eq!(s, "a,123,true");

		// Test with quoting needed
		let res = eval_lua(&lua, r#"return aip.csv.values_to_row({"a,b", 'c "d"'})"#)?;
		let s = res.as_str().ok_or("Should be string")?;
		assert_eq!(s, "\"a,b\",\"c \"\"d\"\"\"");

		// Test with custom options
		let res = eval_lua(&lua, r#"return aip.csv.values_to_row({"a", "b"}, {delimiter = ";"})"#)?;
		let s = res.as_str().ok_or("Should be string")?;
		assert_eq!(s, "a;b");

		// Test with table (json)
		let res = eval_lua(&lua, r#"return aip.csv.values_to_row({"a", {b=1}})"#)?;
		let s = res.as_str().ok_or("Should be string")?;
		// JSON for {b=1} is {"b":1} usually
		assert!(s.starts_with("a,"));
		assert!(s.contains(r#"{""b"":1}"#));

		Ok(())
	}

	#[tokio::test]
	async fn test_aip_csv_values_to_row_special_chars() -> Result<()> {
		let lua = setup_lua(aip_csv::init_module, "csv").await?;

		// Newlines, floats, empty strings
		let script = r#"
			local val = {"line\nbreak", 12.34, ""}
			return aip.csv.values_to_row(val)
		"#;
		let res = eval_lua(&lua, script)?;
		let s = res.as_str().ok_or("Should be string")?;

		// "line\nbreak",12.34,
		assert_eq!(s, "\"line\nbreak\",12.34,");

		Ok(())
	}

	#[tokio::test]
	async fn test_aip_csv_values_to_row_error() -> Result<()> {
		let lua = setup_lua(aip_csv::init_module, "csv").await?;

		let script = r#"
			local val = {"a", function() end}
			return aip.csv.values_to_row(val)
		"#;
		let res = eval_lua(&lua, script);
		assert!(res.is_err());
		let err = res.err().unwrap();
		assert!(err.to_string().contains("unsupported value type 'function'"));

		Ok(())
	}

	#[tokio::test]
	async fn test_aip_csv_value_lists_to_rows() -> Result<()> {
		let lua = setup_lua(aip_csv::init_module, "csv").await?;

		let script = r#"
			local lists = {
				{"a", 1},
				{"b,c", 2}
			}
			return aip.csv.value_lists_to_rows(lists)
		"#;
		let res = eval_lua(&lua, script)?;
		let rows = res.as_array().ok_or("not an array")?;
		assert_eq!(rows.len(), 2);

		let r1 = rows[0].as_str().ok_or("Should have at least one row")?;
		assert_eq!(r1, "a,1");

		let r2 = rows[1].as_str().ok_or("Should have second row")?;
		assert_eq!(r2, "\"b,c\",2");

		Ok(())
	}

	#[tokio::test]
	async fn test_aip_csv_value_lists_to_rows_with_options() -> Result<()> {
		let lua = setup_lua(aip_csv::init_module, "csv").await?;

		let script = r#"
			local lists = {
				{"a", 1},
				{"b", 2}
			}
			return aip.csv.value_lists_to_rows(lists, {delimiter = "|"})
		"#;
		let res = eval_lua(&lua, script)?;
		let rows = res.as_array().ok_or("not an array")?;
		assert_eq!(rows.len(), 2);

		let r1 = rows[0].as_str().ok_or("Should have at least one row")?;
		assert_eq!(r1, "a|1");

		let r2 = rows[1].as_str().ok_or("Should have second row")?;
		assert_eq!(r2, "b|2");

		Ok(())
	}

	#[tokio::test]
	async fn test_aip_csv_parse_with_header_labels() -> Result<()> {
		let lua = setup_lua(aip_csv::init_module, "csv").await?;

		let script = r#"
			local content = "ID,Full Name,Age\n1,Alice,30"
			local opts = {
				header_labels = {
					id = "ID",
					name = "Full Name"
				}
			}
			return aip.csv.parse(content, opts)
		"#;
		let res = eval_lua(&lua, script)?;

		// Verify headers remapping
		assert_eq!(res.x_get_str("/headers/0")?, "id");
		assert_eq!(res.x_get_str("/headers/1")?, "name");
		assert_eq!(res.x_get_str("/headers/2")?, "Age"); // Unmapped

		// Verify rows content remains valid
		assert_eq!(res.x_get_str("/rows/0/0")?, "1");
		assert_eq!(res.x_get_str("/rows/0/1")?, "Alice");
		assert_eq!(res.x_get_str("/rows/0/2")?, "30");

		Ok(())
	}
}

// endregion: --- Tests
