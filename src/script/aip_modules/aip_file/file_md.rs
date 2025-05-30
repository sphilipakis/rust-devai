//! Defines the `file_md` module for the Lua engine.
//!
//! ---
//!
//! ## Lua Documentation
//!
//! The `file_md` module exposes functions to load and convert Markdown content.
//!
//! ### Functions
//!
//! - `aip.file.load_md_sections(path: string, headings?: string | list<string>): list<MdSection>`  
//! - `aip.file.load_md_split_first(path: string): MdSplitFirst`  
//! - `aip.file.save_html_to_md(html_path: string, dest?: string | table): FileInfo`  
//!
//! ## Error
//!
//! Returns an error if file I/O, parsing, or conversion fails.
//
// region:    --- Modules

use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::support::into_vec_of_strings;
use crate::support::md::MdSectionIter;
use mlua::{IntoLua, Lua, Value};

// endregion: --- Modules

/// ## Lua Documentation
///
/// Loads markdown sections from a file, optionally filtering by specific heading names.
///
/// ```lua
/// -- API Signature
/// aip.file.load_md_sections(
///   path: string,
///   headings?: string | list<string>
/// ): list<MdSection>
/// ```
pub(super) fn file_load_md_sections(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	headings: Option<Value>,
) -> mlua::Result<Value> {
	let headings = headings
		.map(|headings| into_vec_of_strings(headings, "file::load_md_sections headings argument"))
		.transpose()?;
	let headings: Option<Vec<&str>> = headings
		.as_deref()
		.map(|vec| vec.iter().map(|s| s.as_str()).collect::<Vec<&str>>());

	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), path.into(), PathResolver::WksDir, None)?;
	let sec_iter = MdSectionIter::from_path(path, headings.as_deref())?;
	let sections = sec_iter.collect::<Vec<_>>();
	sections.into_lua(lua)
}

/// ## Lua Documentation
///
/// Splits a markdown file into three parts based on the *first* heading encountered.
///
/// ```lua
/// -- API Signature
/// aip.file.load_md_split_first(path: string): MdSplitFirst
/// ```
pub(super) fn file_load_md_split_first(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), path.into(), PathResolver::WksDir, None)?;
	let mut sec_iter = MdSectionIter::from_path(path, None)?;
	let split_first = sec_iter.split_first();
	split_first.into_lua(lua)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{assert_contains, assert_not_contains, run_reflective_agent};
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_lua_file_load_md_sections_heading_1_top_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/md-sections.md";

		// -- Exec
		let mut res = run_reflective_agent(
			&format!(r##"return aip.file.load_md_sections("{fx_path}", {{"# Heading 1   "}})"##),
			None,
		)
		.await?;

		// -- Check
		let first_item = res
			.as_array_mut()
			.ok_or("Res should be array")?
			.iter_mut()
			.next()
			.ok_or("Should have at least one item")?;

		let content = first_item.x_get_str("content")?;
		let heading_content = first_item.x_get_str("/heading_content")?;
		let heading_level = first_item.x_get_i64("/heading_level")?;
		let heading_name = first_item.x_get_str("/heading_name")?;
		assert_eq!(heading_level, 1, "heading level");
		assert_contains(heading_content, "# Heading 1");
		assert_contains(heading_name, "Heading 1");
		assert_contains(content, "heading-1-content");
		assert_contains(content, "sub heading 1-a");
		assert_contains(content, "heading-1-a-blockquote");
		assert_not_contains(content, "content-2");
		assert_not_contains(content, "heading-2-blockquote");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_md_split_first_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/md-sections.md";

		// -- Exec
		let res = run_reflective_agent(&format!(r##"return aip.file.load_md_split_first("{fx_path}")"##), None).await?;

		// -- Check
		let before = res.x_get_str("/before")?;
		assert_eq!(before, "", "before should be empty");
		assert_eq!(
			res.x_get_str("/first/heading_content")?,
			"",
			"heading_content should be empty"
		);
		assert_eq!(res.x_get_i64("/first/heading_level")?, 0, "heading level should be 0");
		let content = res.x_get_str("/first/content")?;
		assert_contains(content, "Some early text");
		assert_contains(content, "- and more early text");
		assert_not_contains(content, "# Heading 1");
		assert_not_contains(content, "Some heading-1-content");
		let after = res.x_get_str("/after")?;
		assert_contains(after, "# Heading 1");
		assert_contains(after, "Some heading-1-content");
		assert_contains(after, "# Heading three");

		Ok(())
	}
}

// endregion: --- Tests
