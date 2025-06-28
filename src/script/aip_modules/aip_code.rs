//! Defines the `code` module for the Lua engine.
//!
//! This module provides utility functions for processing and formatting code,
//! particularly for generating commented lines based on language extensions.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.code` module exposes functions that process code formatting.
//!
//! ### Functions
//!
//! - `aip.code.comment_line(lang_ext: string, comment_content: string): string`

use crate::Result;
use mlua::{Lua, Table};

pub fn init_module(lua: &Lua, _runtime: &crate::runtime::Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	table.set("comment_line", lua.create_function(comment_line)?)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Creates a comment line based on the language extension and the given comment content.
///
/// ```lua
/// -- API Signature
/// aip.code.comment_line(lang_ext: string, comment_content: string): string
/// ```
///
/// Takes a language extension and content string, and returns a formatted comment line
/// appropriate for that language.
///
/// ### Arguments
///
/// - `lang_ext: string`: A string representing the file extension or language identifier (e.g., "rs", "lua", "py"). Case-insensitive.
/// - `comment_content: string`: The content that should be commented.
///
/// ### Returns
///
/// - `string`: A string representing the commented line without a trailing newline.
///
/// ```ts
/// string  // A string representing the commented line without a trailing newline.
/// ```
///
/// ### Error
///
/// Returns an error if any error occurs during the Lua-Rust conversion or string formatting.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn comment_line(_lua: &Lua, (lang_ext, comment_content): (String, String)) -> mlua::Result<String> {
	// Normalize the language extension by trimming and converting to lowercase.
	let ext = lang_ext.trim().to_lowercase();
	let comment = match ext.as_str() {
		"lua" | "sql" => format!("-- {comment_content}"),
		"html" => format!("<!-- {comment_content} -->"),
		"css" | "pcss" => format!("/* {comment_content} */"),
		"py" => format!("# {comment_content}"),
		_ => format!("// {comment_content}"),
	};
	Ok(comment)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};

	#[tokio::test]
	async fn test_code_comment_line_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "code")?;

		// Define test cases as tuples: (language extension, comment content, expected result)
		let test_cases = [
			("rs", "Rust comment", "// Rust comment"),
			("lua", "Lua comment", "-- Lua comment"),
			("sql", "SQL comment", "-- SQL comment"),
			("html", "HTML comment", "<!-- HTML comment -->"),
			("css", "CSS comment", "/* CSS comment */"),
			("pcss", "PCSS comment", "/* PCSS comment */"),
			("py", "Python comment", "# Python comment"),
			("js", "JavaScript comment", "// JavaScript comment"),
			// Test case with leading/trailing whitespace and different casing
			("  Rs ", "  Another Rust comment  ", "//   Another Rust comment  "),
		];

		// -- Exec & Check
		for (lang, content, expected) in test_cases.iter() {
			let script = format!("return aip.code.comment_line({lang:?}, {content:?})");
			let res = eval_lua(&lua, &script)?;
			let res_str = res.as_str().ok_or("Expected a string result")?;
			assert_eq!(
				res_str, *expected,
				"Failed for lang_ext: {lang} with content: {content}"
			);
		}
		Ok(())
	}
}

// endregion: --- Tests
