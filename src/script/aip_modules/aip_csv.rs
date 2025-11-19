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
//!
//! - `aip.csv.parse(content: string, options?: CsvOptions): { headers: string[] | nil, rows: string[][] }`
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

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let parse_row_fn =
		lua.create_function(|lua, (row, opts): (String, Option<Value>)| lua_parse_row(lua, row, opts))?;
	let parse_fn =
		lua.create_function(|lua, (content, opts): (String, Option<Value>)| lua_parse(lua, content, opts))?;
	let values_to_row_fn = lua.create_function(|lua, values: Value| lua_values_to_row(lua, values))?;

	table.set("parse_row", parse_row_fn)?;
	table.set("parse", parse_fn)?;
	table.set("values_to_row", values_to_row_fn)?;

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
/// Returns a table with `headers` (or nil) and `rows` (string[][]).
/// By default this API treats the first row as headers (`has_header = true`) and skips empty lines
/// (`skip_empty_lines = true`) unless these options are overridden.
///
/// ```lua
/// -- API Signature
/// aip.csv.parse(content: string, options?: CsvOptions): { headers: string[] | nil, rows: string[][] }
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
/// aip.csv.values_to_row(values: any[]): string
/// ```
///
/// - `values`: A list of values (strings, numbers, booleans, nil, or tables).
///   - Tables are converted to JSON strings.
///   - Nil (and null sentinel) are converted to empty strings.
fn lua_values_to_row(_lua: &Lua, values: Value) -> mlua::Result<String> {
	let table = match values {
		Value::Table(t) => t,
		_ => {
			return Err(mlua::Error::external(
				"aip.csv.values_to_row - values must be a table (list)",
			));
		}
	};

	let mut row_values = Vec::new();

	// Iterate over sequence values (1..N)
	for value in table.sequence_values::<Value>() {
		let value = value?;
		let s = match value {
			Value::String(s) => s.to_str()?.to_string(),
			Value::Integer(i) => i.to_string(),
			Value::Number(n) => n.to_string(),
			Value::Boolean(b) => b.to_string(), // "true" or "false"
			Value::Nil => "".to_string(),
			Value::UserData(ud) if ud.is::<crate::script::NullSentinel>() => "".to_string(),
			Value::Table(t) => {
				let serde_val = lua_value_to_serde_value(Value::Table(t)).map_err(mlua::Error::external)?;
				serde_json::to_string(&serde_val).map_err(mlua::Error::external)?
			}
			other => {
				return Err(mlua::Error::external(format!(
					"aip.csv.values_to_row - unsupported value type '{}'",
					other.type_name()
				)));
			}
		};
		row_values.push(s);
	}

	let csv_row = crate::support::csvs::values_to_csv_row(&row_values).map_err(mlua::Error::external)?;
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
		assert_eq!(s, "a,123,true,");

		// Test with quoting needed
		let res = eval_lua(&lua, r#"return aip.csv.values_to_row({"a,b", 'c "d"'})"#)?;
		let s = res.as_str().ok_or("Should be string")?;
		assert_eq!(s, "\"a,b\",\"c \"\"d\"\"\"");

		// Test with table (json)
		let res = eval_lua(&lua, r#"return aip.csv.values_to_row({"a", {b=1}})"#)?;
		let s = res.as_str().ok_or("Should be string")?;
		// JSON for {b=1} is {"b":1} usually
		assert!(s.starts_with("a,"));
		assert!(s.contains(r#"{""b"":1}"#));

		Ok(())
	}
}

// endregion: --- Tests
