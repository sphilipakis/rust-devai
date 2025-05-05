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
/// Load markdown sections from a file, optionally filtering by specific heading names.
///
/// ```lua
/// -- API Signature
/// aip.file.load_md_sections(
///   path: string,
///   headings?: string | list<string>
/// ): list<MdSection>
/// ```
///
/// Reads the markdown file specified by `path` (relative to the workspace root)
/// and splits it into sections based on markdown headings (lines starting with `#`).
/// It returns a list of `MdSection` objects.
///
/// If the optional `headings` argument is provided (as a single heading string or a list of heading strings),
/// only the sections corresponding exactly to those heading names (case-sensitive, excluding the `#` markers and leading/trailing whitespace)
/// will be returned. If `headings` is omitted or nil, all sections (including content before the first heading, if any) are returned.
///
/// ### Arguments
///
/// - `path: string`: The path to the markdown file, relative to the workspace root.
/// - `headings?: string | list<string>` (optional): A single heading name (e.g., "Introduction") or a Lua list (table)
///   of heading names to filter the sections by. The matching is case-sensitive and ignores the `#` marker and surrounding whitespace.
///
/// ### Returns
///
/// - `list<MdSection>: table`: A Lua list (table) where each element is an `MdSection` table:
///   ```ts
///   {
///     content: string,    // The full content of the section, including sub-headings and their content.
///                         // For the first section before any heading, this is the initial content.
///     heading?: {         // Present if the section has a heading (absent for content before the first heading).
///       content: string,  // The raw heading line (e.g., "## Section Title").
///       level: number,    // Heading level (1 for #, 2 for ##, etc.).
///       name: string      // The extracted heading name (e.g., "Section Title").
///     }
///   }
///   ```
///   The list is empty if the file is empty or if filtering is applied and no matching sections are found.
///
/// ### Example
///
/// ```lua
/// -- Assuming 'doc/readme.md' has sections:
/// -- (no heading)
/// -- # Summary
/// -- ## Details
/// -- # Conclusion
///
/// -- Load all sections
/// local all_sections = aip.file.load_md_sections("doc/readme.md")
/// print(#all_sections) -- Output: 4 (includes content before first heading)
/// print(all_sections[2].heading.name) -- Output: "Summary"
///
/// -- Load only the "Summary" section
/// local summary_section = aip.file.load_md_sections("doc/readme.md", "Summary")
/// print(#summary_section) -- Output: 1
/// print(summary_section[1].heading.name) -- Output: "Summary"
///
/// -- Load "Summary" and "Conclusion" sections
/// local sections = aip.file.load_md_sections("doc/readme.md", {"Summary", "Conclusion"})
/// print(#sections) -- Output: 2
/// print(sections[1].heading.name) -- Output: "Summary"
/// print(sections[2].heading.name) -- Output: "Conclusion"
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The file specified by `path` cannot be found or read.
/// - The `headings` argument is provided but is not a string or a list of strings.
/// - An error occurs during markdown parsing or conversion to Lua values.
///
/// ```ts
/// {
///   error: string // Error message
/// }
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
/// Splits a markdown file into three parts based on the *first* heading encountered.
///
/// ```lua
/// -- API Signature
/// aip.file.load_md_split_first(path: string): MdSplitFirst
/// ```
///
/// Reads the markdown file specified by `path` (relative to the workspace root) and divides its content:
/// 1.  `before`: All content from the beginning of the file up to (but not including) the first line that constitutes a markdown heading (starts with `#`).
/// 2.  `first`: The `MdSection` object representing the first heading encountered and all content following it until the next heading of the same or lower level, or the end of the file.
/// 3.  `after`: All remaining content from the start of the *next* heading (if any) to the end of the file.
///
/// If the file contains no headings, `before` will contain the entire file content, and `first` and `after` will represent empty/default sections.
/// If the file starts directly with a heading, `before` will be an empty string.
///
/// ### Arguments
///
/// - `path: string`: The path to the markdown file, relative to the workspace root.
///
/// ### Returns
///
/// - `MdSplitFirst: table`: A table containing the three parts:
///   ```ts
///   {
///     before: string,       // Content before the first heading. Empty if file starts with a heading.
///     first: {              // The first MdSection encountered (or a default one if no headings).
///       content: string,    // Content belonging to the first heading.
///       heading?: {         // Heading information (absent if no headings in file).
///         content: string,  // Raw heading line.
///         level: number,    // Heading level.
///         name: string      // Extracted heading name.
///       }
///     },
///     after: string         // All content from the second heading onwards. Empty if only one or zero headings.
///   }
///   ```
///
/// ### Example
///
/// ```lua
/// -- Assuming 'doc/structure.md' contains:
/// -- Some introductory text.
/// -- More intro.
/// --
/// -- # Chapter 1
/// -- Content for chapter 1.
/// -- ## Section 1.1
/// -- Details for 1.1.
/// -- # Chapter 2
/// -- Content for chapter 2.
///
/// local split = aip.file.load_md_split_first("doc/structure.md")
///
/// print("--- BEFORE ---")
/// print(split.before)
/// -- Output:
/// -- Some introductory text.
/// -- More intro.
/// --
///
/// print("--- FIRST Heading ---")
/// print(split.first.heading.name) -- Output: Chapter 1
/// print("--- FIRST Content ---")
/// print(split.first.content)
/// -- Output: (Content including "# Chapter 1", "Content for chapter 1.", "## Section 1.1", "Details for 1.1.")
/// -- Note: Content includes the heading line itself and sub-sections.
///
/// print("--- AFTER ---")
/// print(split.after)
/// -- Output: (Content including "# Chapter 2", "Content for chapter 2.")
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The file specified by `path` cannot be found or read.
/// - An error occurs during markdown parsing or conversion to Lua values.
///
/// ```ts
/// {
///   error: string // Error message
/// }
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
