use crate::script::LuaValueExt;
use mlua::{FromLua, Lua, Value};
use simple_fs::SPath;

/// Destination options for the `aip.file.save_html_to_md` API.
///
/// This argument can be:
///
/// - `nil`: The Markdown file path is derived from the source HTML path
///   by replacing its extension with `.md`.
///   e.g., `index.html` → `index.md`.
///
/// - `string`: A path (relative or absolute) specifying where to save the Markdown file.
///
/// - `table`: A custom options table with any of the following optional fields:
///     - `base_dir: string`
///       Base directory (workspace-relative) to resolve the destination path.
///     - `file_name: string`
///       Custom file name (with or without extension) for the Markdown output.
///     - `suffix: string`
///       Suffix appended to the source file stem before the `.md` extension.
///       e.g., with `suffix = "_v2"`, `index.html` → `index_v2.md`.
///     - `slim: boolean`
///       If `true`, slim the HTML content (remove scripts, styles, comments) before converting to Markdown. Defaults to `false`.
///
/// ```lua
/// -- Example usages
/// -- Default behavior (nil):
/// aip.file.save_html_to_md("docs/page.html", nil)
///
/// -- Destination as string:
/// aip.file.save_html_to_md("docs/page.html", "out/custom.md")
///
/// -- Custom options table:
/// aip.file.save_html_to_md("docs/page.html", {
///     base_dir = "output",
///     file_name = "renamed.md",
///     suffix = "_v2",
///     slim = true,
/// })
/// ```
pub enum DestOptions {
	/// `nil`: use source path with `.md` extension
	Nil,
	/// `string`: exact path for the Markdown file
	Path(SPath),
	/// custom options table
	Custom(OptionsCustom),
}

/// Custom destination fields for `DestOptions::Custom`.
pub struct OptionsCustom {
	/// Directory to resolve the destination path against (workspace-relative).
	pub base_dir: Option<SPath>,
	/// Custom file name for the Markdown output.
	pub file_name: Option<String>,
	/// Suffix appended to the source file stem before `.md`.
	pub suffix: Option<String>,
	/// If `true`, slims the HTML before converting to markdown.
	pub slim: Option<bool>,
}

impl FromLua for DestOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(DestOptions::Nil),

			Value::String(s) => {
				let path_str = s.to_string_lossy();
				Ok(DestOptions::Path(SPath::new(path_str)))
			}

			Value::Table(table) => {
				let base_dir = table.x_get_string("base_dir").map(|s| SPath::new(&s));
				let file_name = table.x_get_string("file_name");
				let suffix = table.x_get_string("suffix");
				let slim = table.x_get_bool("slim");

				Ok(DestOptions::Custom(OptionsCustom {
					base_dir,
					file_name,
					suffix,
					slim,
				}))
			}

			other => Err(mlua::Error::FromLuaConversionError {
				from: other.type_name(),
				to: "DestOptions".to_string(),
				message: Some(
					"Destination Options argument can be nil, a string path, or { base_dir?, file_name?, suffix?, slim? }"
						.into(),
				),
			}),
		}
	}
}
