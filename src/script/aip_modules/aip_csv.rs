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

use crate::runtime::Runtime;
use crate::types::{CsvContent, CsvOptions};
use crate::{Error, Result};
use mlua::{FromLua as _, IntoLua as _, Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let parse_row_fn =
		lua.create_function(|lua, (row, opts): (String, Option<Value>)| lua_parse_row(lua, row, opts))?;
	let parse_fn =
		lua.create_function(|lua, (content, opts): (String, Option<Value>)| lua_parse(lua, content, opts))?;

	table.set("parse_row", parse_row_fn)?;
	table.set("parse", parse_fn)?;

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

	// TODO: Needs to handle if row is empty
	let mut builder = opts.into_reader_builder();
	builder.has_headers(false).comment(None).flexible(true);
	let mut rdr = builder.from_reader(row.as_bytes());

	let rec_opt = rdr
		.records()
		.next()
		.transpose()
		.map_err(|e| Error::custom(format!("aip.csv.parse_row failed. {e}")))?;

	match rec_opt {
		Some(rec) => string_record_to_lua_array(lua, &rec),
		None => lua.create_table(), // empty list
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

	// Default to has_header = true for this API; empty lines are skipped by default.
	let has_header = opts.has_header.unwrap_or(true);
	let skip_empty = opts.skip_empty_lines.unwrap_or(true);

	let mut builder = opts.into_reader_builder();
	builder.has_headers(has_header).flexible(true);

	let mut rdr = builder.from_reader(content.as_bytes());

	let mut headers = Vec::new();

	// headers
	if has_header {
		let hdr = rdr
			.headers()
			.map_err(|e| Error::custom(format!("aip.csv.parse failed to read headers. {e}")))?;
		headers = string_record_to_vec(hdr);
	}

	// rows
	let mut rows = Vec::new();

	for rec_res in rdr.records() {
		let rec = rec_res.map_err(|e| Error::custom(format!("aip.csv.parse failed to read record. {e}")))?;
		if skip_empty && is_record_empty(&rec) {
			continue;
		}
		rows.push(string_record_to_vec(&rec));
	}

	let csv_content = CsvContent { headers, rows };

	csv_content.into_lua(lua)
}

// endregion: --- Lua Fns

// region:    --- Support

fn string_record_to_lua_array(lua: &Lua, rec: &csv::StringRecord) -> mlua::Result<Table> {
	let t = lua.create_table()?;
	for (i, field) in rec.iter().enumerate() {
		t.set(i + 1, field)?;
	}
	Ok(t)
}

fn string_record_to_vec(rec: &csv::StringRecord) -> Vec<String> {
	rec.iter().map(|s| s.to_string()).collect()
}

fn is_record_empty(rec: &csv::StringRecord) -> bool {
	rec.iter().all(|s| s.trim().is_empty())
}

// endregion: --- Support

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
}

// endregion: --- Tests
