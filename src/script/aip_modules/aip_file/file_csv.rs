//! Lua CSV helpers for `aip.file`.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` CSV helpers
//!
//! ### Functions
//!
//! - `aip.file.load_csv_headers(path: string): string[]`
//! - `aip.file.load_csv(path: string, with_headers?: boolean): { headers: string[], content: string[][] }`
//!
//! The `path` is resolved relative to the workspace root.

use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::support::W;
use mlua::{IntoLua, Lua, Value};

/// ## Lua Documentation
///
/// Loads a CSV file and returns its header row as a list of strings.
///
/// ```lua
/// -- API Signature
/// aip.file.load_csv_headers(path: string): string[]
/// ```
///
/// - `path: string` — CSV file path, relative to the workspace root (supports pack refs).
///
/// Returns a Lua array (list) of strings containing the header names.
///
/// ### Example
///
/// ```lua
/// local headers = aip.file.load_csv_headers("data/example.csv")
/// for i, h in ipairs(headers) do
///   print(i, h)
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The path cannot be resolved,
/// - The file cannot be found or read,
/// - CSV parsing fails.
pub(super) fn file_load_csv_headers(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	let headers = crate::support::files::load_csv_headers(&full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.load_csv_headers - Failed to read csv file '{path}'.\nCause: {e}",
		))
	})?;

	let headers_tbl = W(headers).into_lua(lua)?;
	Ok(headers_tbl)
}

/// ## Lua Documentation
///
/// Loads a CSV file and returns its headers (optionally) and all rows as string arrays.
///
/// ```lua
/// -- API Signature
/// aip.file.load_csv(path: string, with_headers?: boolean): { headers: string[], content: string[][] }
/// ```
///
/// - `path: string` — CSV file path, relative to the workspace root (supports pack refs).
/// - `with_headers?: boolean` — When `true` (default), the first row is treated as headers and
///   not included in `content`. When `false`, no header parsing is performed and the `headers`
///   field will be an empty array.
///
/// ### Returns
///
/// - `{ headers: string[], content: string[][] }`
///
/// ### Example
///
/// ```lua
/// local res = aip.file.load_csv("data/example.csv") -- defaults to with_headers = true
/// print("Headers:", table.concat(res.headers, ", "))
/// for _, row in ipairs(res.content) do
///   print(table.concat(row, " | "))
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The path cannot be resolved,
/// - The file cannot be found or read,
/// - CSV parsing fails.
pub(super) fn file_load_csv(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	with_headers: Option<bool>,
) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	let resp = crate::support::files::load_csv(&full_path, with_headers).map_err(|e| {
		Error::from(format!(
			"aip.file.load_csv - Failed to read csv file '{path}'.\nCause: {e}",
		))
	})?;

	let headers_tbl = W(resp.headers).into_lua(lua)?;
	let content_tbl = W(resp.content).into_lua(lua)?;
	// let content_tbl = crate::script::support::lua_array_of_string_arrays(lua, resp.content)?;

	let res_tbl = lua.create_table()?;
	res_tbl.set("headers", headers_tbl)?;
	res_tbl.set("content", content_tbl)?;

	Ok(Value::Table(res_tbl))
}
