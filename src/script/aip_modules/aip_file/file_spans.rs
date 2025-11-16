//! Lua span helpers for `aip.file`.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` span helpers
//!
//! ### Functions
//!
//! - `aip.file.line_spans(path: string): [start,end][]`
//! - `aip.file.csv_row_spans(path: string): [start,end][]`
//! - `aip.file.read_span(path: string, start: integer, end: integer): string`
//!
//! The `path` is resolved relative to the workspace root.

use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use mlua::{Lua, Value};
use simple_fs::{self, SPath};

// region:    --- Lua Spans

/// ## Lua Documentation
///
/// Returns the byte spans for each line in a text file resolved from the workspace root.
///
/// ```lua
/// -- API Signature
/// aip.file.line_spans(path: string): [start,end][]
/// ```
///
/// - `path: string`: File path relative to the workspace root (pack refs supported).
///
/// The resulting Lua table contains `[start, end]` pairs (1-indexed array) describing each line's
/// byte range in the source file.
///
/// ### Example
///
/// ```lua
/// local spans = aip.file.line_spans("notes/todo.txt")
/// for i, span in ipairs(spans) do
///   print(("line %d: %d -> %d"):format(i, span[1], span[2]))
/// end
/// ```
///
/// ### Error
///
/// Returns an error if the path cannot be resolved, the file cannot be read, or line spans cannot be computed.
pub(super) fn file_line_spans(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let rel_path = SPath::new(path);
	let full_path = runtime
		.dir_context()
		.resolve_path(runtime.session(), rel_path, PathResolver::WksDir, None)?;

	let spans = simple_fs::line_spans(&full_path).map_err(Error::from)?;
	let table = spans_to_lua_table(lua, &spans)?;
	Ok(Value::Table(table))
}

/// ## Lua Documentation
///
/// Returns the byte spans for each CSV row of the given file.
///
/// ```lua
/// -- API Signature
/// aip.file.csv_row_spans(path: string): [start,end][]
/// ```
///
/// - `path: string`: CSV file path relative to the workspace root (pack refs supported).
///
/// The returned Lua table contains `[start, end]` byte offset pairs describing each row, which is helpful
/// when mapping Lua data back to exact CSV locations.
///
/// ### Example
///
/// ```lua
/// local spans = aip.file.csv_row_spans("data/example.csv")
/// local first = spans[1]
/// print(("row 1 bytes: %d -> %d"):format(first[1], first[2]))
/// ```
///
/// ### Error
///
/// Returns an error if the path cannot be resolved, the file is missing, or row spans cannot be computed.
pub(super) fn file_csv_row_spans(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let rel_path = SPath::new(path);
	let full_path = runtime
		.dir_context()
		.resolve_path(runtime.session(), rel_path, PathResolver::WksDir, None)?;

	let spans = simple_fs::csv_row_spans(&full_path).map_err(Error::from)?;
	let table = spans_to_lua_table(lua, &spans)?;
	Ok(Value::Table(table))
}

/// ## Lua Documentation
///
/// Reads a byte range from a file and returns the captured UTF-8 string.
///
/// ```lua
/// -- API Signature
/// aip.file.read_span(path: string, start: integer, end: integer): string
/// ```
///
/// - `path: string`: File path relative to the workspace root (pack refs supported).
/// - `start: integer`: Inclusive byte offset where reading begins (zero-based).
/// - `end: integer`: Exclusive byte offset where reading stops (zero-based).
///
/// The function returns the substring defined by `[start, end)` which is useful alongside the span helpers.
///
/// ### Example
///
/// ```lua
/// local spans = aip.file.line_spans("notes/todo.txt")
/// local snippet = aip.file.read_span("notes/todo.txt", spans[1][1], spans[1][2])
/// print(snippet)
/// ```
///
/// ### Error
///
/// Returns an error if the offsets are negative, `end` is smaller than `start`, the path cannot be resolved,
/// the file is missing, or the requested slice cannot be read.
pub(super) fn file_read_span(
	_lua: &Lua,
	runtime: &Runtime,
	path: String,
	start: i64,
	end: i64,
) -> mlua::Result<String> {
	if start < 0 || end < 0 {
		return Err(Error::custom("read_span expects non-negative start/end offsets").into());
	}
	if end < start {
		return Err(Error::custom("read_span expects end >= start").into());
	}

	let rel_path = SPath::new(path);
	let full_path = runtime
		.dir_context()
		.resolve_path(runtime.session(), rel_path, PathResolver::WksDir, None)?;

	let text = simple_fs::read_span(&full_path, start as usize, end as usize).map_err(Error::from)?;
	Ok(text)
}

// endregion: --- Lua Spans

// region:    --- Support

fn spans_to_lua_table(lua: &Lua, spans: &[(usize, usize)]) -> mlua::Result<mlua::Table> {
	let out = lua.create_table()?;
	for (i, (start, end)) in spans.iter().enumerate() {
		let row = lua.create_table()?;
		row.set(1, *start as i64)?;
		row.set(2, *end as i64)?;
		out.set(i + 1, row)?;
	}
	Ok(out)
}

// endregion: --- Support
