//! Defines the `file_md` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `file_md` module exposes functions to load and parse markdown files.
//!
//! ### Functions
//!
//! - `aip.file.load_md_sections(path: string, headings?: string | list): list`
//! - `aip.file.load_md_split_first(path: string): {before: string, first: {content: string, heading: {content: string, level: number, name: string}}, after: string}`

use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::lua_script::helpers::to_vec_of_strings;
use crate::support::md::MdSectionIter;
use mlua::{IntoLua, Lua, Value};

/// ## Lua Documentation
///
/// Load markdown sections from a file, optionally filtering by headings.
///
/// ```lua
/// -- API Signature
/// aip.file.load_md_sections(path: string, headings?: string | list): list
/// ```
///
/// ### Arguments
///
/// - `path: string`: Path to the markdown file.
/// - `headings?: string | list`: Optional string or list of strings representing the headings to filter by.
///
/// ### Returns
///
/// ```ts
/// -- Array/Table of MdSection
/// {
///   content: string,    // Content of the section
///   heading?: {         // Heading information (optional)
///     content: string,  // Heading content
///     level: number,    // Heading level (e.g., 1 for #, 2 for ##)
///     name: string      // Heading name
///   }
/// }
/// ```
///
/// ### Example
///
/// ```lua
/// -- Load all sections from a file
/// local all_sections = aip.file.load_md_sections("doc/readme.md")
///
/// -- Load only sections with the heading "# Summary"
/// local summary_section = aip.file.load_md_sections("doc/readme.md", "# Summary")
///
/// -- Load sections with multiple headings
/// local sections = aip.file.load_md_sections("doc/readme.md", {"# Summary", "## Details"})
/// ```
pub(super) fn file_load_md_sections(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	headings: Option<Value>,
) -> mlua::Result<Value> {
	let headings = headings
		.map(|headings| to_vec_of_strings(headings, "file::load_md_sections headings argument"))
		.transpose()?;
	let headings: Option<Vec<&str>> = headings.as_deref().map(|vec| {
		// Create a slice of string references directly from the vector.
		vec.iter().map(|s| s.as_str()).collect::<Vec<&str>>()
	});

	let path = runtime.dir_context().resolve_path(path.into(), PathResolver::WksDir)?;

	let sec_iter = MdSectionIter::from_path(path, headings.as_deref())?;
	let sections = sec_iter.collect::<Vec<_>>();
	let res = sections.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Splits a markdown file into three parts: content before the first heading, the first heading and its content, and the rest of the file.
///
/// ```lua
/// -- API Signature
/// aip.file.load_md_split_first(path: string): {before: string, first: {content: string, heading: {content: string, level: number, name: string}}, after: string}
/// ```
///
/// ### Arguments
///
/// - `path: string`: Path to the markdown file.
///
/// ### Returns
///
/// ```ts
/// {
///   before: string,       // Content before the first heading
///   first: {              // Information about the first section
///     content: string,    // Content of the first section
///     heading: {          // Heading of the first section
///       content: string,  // Heading content
///       level: number,    // Heading level (e.g., 1 for #, 2 for ##)
///       name: string      // Heading name
///     }
///   },
///   after: string         // Content after the first section
/// }
/// ```
///
/// ### Example
///
/// ```lua
/// local split = aip.file.load_md_split_first("doc/readme.md")
/// print(split.before)        -- Content before the first heading
/// print(split.first.content) -- Content of the first section
/// print(split.after)         -- Content after the first section
/// ```
pub(super) fn file_load_md_split_first(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let path = runtime.dir_context().resolve_path(path.into(), PathResolver::WksDir)?;

	let mut sec_iter = MdSectionIter::from_path(path, None)?;
	let split_first = sec_iter.split_first();

	let res = split_first.into_lua(lua)?;

	Ok(res)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

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
		// contains
		assert_contains(heading_content, "# Heading 1");
		assert_contains(heading_name, "Heading 1");
		assert_contains(content, "heading-1-content");
		assert_contains(content, "sub heading 1-a");
		assert_contains(content, "heading-1-a-blockquote");
		// not contains
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
		// check before
		let before = res.x_get_str("/before")?;
		assert_eq!(before, "", "before should be empty");
		// check first heading
		assert_eq!(
			res.x_get_str("/first/heading_content")?,
			"",
			"heading_content should be empty"
		);
		assert_eq!(res.x_get_i64("/first/heading_level")?, 0, "heading level should be 0");
		// check first content
		let content = res.x_get_str("/first/content")?;
		assert_contains(content, "Some early text");
		assert_contains(content, "- and more early text");
		assert_not_contains(content, "# Heading 1");
		assert_not_contains(content, "Some heading-1-content");
		// check the after
		let after = res.x_get_str("/after")?;
		assert_contains(after, "# Heading 1");
		assert_contains(after, "Some heading-1-content");
		assert_contains(after, "# Heading three");

		Ok(())
	}

	// other/md-sections.md
}

// endregion: --- Tests
