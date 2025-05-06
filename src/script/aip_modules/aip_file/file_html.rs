use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::helpers::to_vec_of_strings;
use crate::support::md::MdSectionIter;

use crate::types::FileMeta;
use mlua::FromLua;
use mlua::{IntoLua, Lua, Value};
use simple_fs::SPath;
use std::fs::{read_to_string, write};

/// ## Lua Documentation
///
/// Loads an HTML file, converts its content to Markdown, and saves it according to `dest` options.
///
/// ```lua
/// -- API Signature
/// aip.file.save_html_to_md(
///   html_path: string,
///   dest?: string | {
///     base_dir?: string,
///     file_name?: string,
///     suffix?: string
///   }
/// ): FileMeta
/// ```
///
/// ### Arguments
///
/// - `html_path: string`  
///   Path to the source HTML file, relative to the workspace root.
///
/// - `dest: string | table (optional)`  
///   Destination path or options table:
///
///   - `string`  
///     Path to save the `.md` file (relative or absolute).
///
///   - `table` (`DestOptions`):
///       - `base_dir?: string`: Base directory for resolving the destination.
///       - `file_name?: string`: Custom file name for the Markdown output.
///       - `suffix?: string`: Suffix appended to the source file stem before `.md`.
///
/// ### Returns
///
/// - `FileMeta`  
///   Metadata about the created Markdown file (path, name, stem, ext, timestamps, size).
///
/// ### Example
///
/// ```lua
/// -- Default (replaces .html with .md):
/// aip.file.save_html_to_md("docs/page.html")
///
/// -- Using a custom string path:
/// aip.file.save_html_to_md("docs/page.html", "out/custom.md")
///
/// -- Using options table:
/// aip.file.save_html_to_md("docs/page.html", {
///   base_dir = "output",
///   suffix = "_v2",
/// })
/// ```
pub(super) fn file_save_html_to_md(
	lua: &Lua,
	runtime: &Runtime,
	html_path: String,
	dest: Value,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();

	// -- resolve and read source
	let rel_html = SPath::new(html_path.clone());
	let full_html = dir_context.resolve_path(rel_html.clone(), PathResolver::WksDir)?;
	let html_content = read_to_string(&full_html)
		.map_err(|e| Error::Custom(format!("Failed to read HTML file '{}'. Cause: {}", html_path, e)))?;

	// -- convert to Markdown
	let md_content = crate::support::html::to_md(html_content).map_err(|e| {
		Error::Custom(format!(
			"Failed to convert HTML file '{}' to Markdown. Cause: {}",
			html_path, e
		))
	})?;

	// -- determine destination paths using the new helper
	let (rel_md, full_md) = super::support::resolve_dest_path(lua, dir_context, &rel_html, dest, "md")?;

	// -- write out and return metadata
	simple_fs::ensure_file_dir(&full_md).map_err(Error::from)?;
	write(&full_md, md_content)
		.map_err(|e| Error::Custom(format!("Failed to write Markdown file '{}'. Cause: {}", rel_md, e)))?;

	let meta = FileMeta::new(rel_md, &full_md);
	meta.into_lua(lua)
}
