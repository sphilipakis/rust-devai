//! Defines the `html` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This module exposes functions that process HTML content.
//!
//! ### Functions
//!
//! - `aip.html.slim(html_content: string) -> string`
//! - `aip.html.to_md(html_content: string) -> string`

use crate::runtime::Runtime;
use crate::{Result, support};
use mlua::{Lua, Table};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let prune_fn = lua.create_function(html_slim)?;
	table.set("slim", prune_fn.clone())?;

	// deprecated (TODO: need to send a deprecation notice once we have the deprecation)
	table.set("prune_to_content", prune_fn)?;

	let to_md_fn = lua.create_function(html_to_md)?;
	table.set("to_md", to_md_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Strips non-content elements from the provided HTML content and returns the cleaned HTML as a string.
///
/// This function removes:
/// - Non-visible tags such as `<script>`, `<link>`, `<style>`, and `<svg>`.
/// - HTML comments.
/// - Empty lines.
/// - Attributes except for `class`, `aria-label`, and `href`.
///
/// ```lua
/// -- API Signature
/// aip.html.slim(html_content: string): string
/// ```
///
/// ### Arguments
///
/// - `html_content: string`: The HTML content to be pruned.
///
/// ### Returns
///
/// `string`: The cleaned HTML content.
///
/// ### Example
///
/// ```lua
/// local cleaned_html = aip.html.slim(html_content)
/// ```
///
/// ### Error
///
/// Returns an error if the HTML content fails to be pruned.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn html_slim(_lua: &Lua, html_content: String) -> mlua::Result<String> {
	support::html::slim(html_content)
		.map_err(|err| mlua::Error::RuntimeError(format!("Failed to prune HTML content: {}", err)))
}

/// ## Lua Documentation
///
/// Converts HTML content to Markdown format.
///
/// ```lua
/// -- API Signature
/// aip.html.to_md(html_content: string): string
/// ```
///
/// ### Arguments
///
/// - `html_content: string`: The HTML content to be converted.
///
/// ### Returns
///
/// `string`: The Markdown representation of the HTML content.
///
/// ### Example
///
/// ```lua
/// local markdown_content = aip.html.to_md("<h1>Hello</h1><p>World</p>")
/// -- markdown_content will be "# Hello\n\nWorld\n"
/// ```
///
/// ### Error
///
/// Returns an error if the HTML content fails to be converted to Markdown.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn html_to_md(_lua: &Lua, html_content: String) -> mlua::Result<String> {
	support::html::to_md(html_content)
		.map_err(|err| mlua::Error::RuntimeError(format!("Failed to convert HTML to Markdown: {}", err)))
}

// region:    --- Tests
#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::lua_script::aip_html;

	#[tokio::test]
	async fn test_lua_html_slim_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_html::init_module, "html")?;
		let fx_script = r#"
local html_content = [[
<!DOCTYPE html>
<html>
<head>
    <script>alert('test');</script>
    <style>body { color: red; }</style>
</head>
<body>
    <div class="content">Hello World</div>
    <!-- comment -->
</body>
</html>
]]
return aip.html.slim(html_content)
        "#;
		// -- Exec
		let res = eval_lua(&lua, fx_script)?;
		// -- Check
		let cleaned_html = res.as_str().unwrap();
		assert!(!cleaned_html.contains("<script>"));
		assert!(!cleaned_html.contains("<style>"));
		assert!(!cleaned_html.contains("<!-- comment -->"));
		assert!(cleaned_html.contains(r#"<div class="content">Hello World</div>"#));
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_html_to_md_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_html::init_module, "html")?;
		let fx_script = r#"
local html_content = "<h1>Title</h1><p>Some <strong>bold</strong> text.</p><ul><li>Item 1</li><li>Item 2</li></ul>"
return aip.html.to_md(html_content)
        "#;

		// -- Exec
		let res = eval_lua(&lua, fx_script)?;

		// -- Check
		let md_content = res.as_str().ok_or("Result should be string")?;
		let expected_md = "# Title\n\nSome **bold** text.\n\n-   Item 1\n-   Item 2";
		assert_eq!(md_content, expected_md);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_html_to_md_empty_input() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_html::init_module, "html")?;
		let fx_script = r#"
return aip.html.to_md("")
        "#;

		// -- Exec
		let res = eval_lua(&lua, fx_script)?;

		// -- Check
		let md_content = res.as_str().unwrap();
		assert_eq!(md_content, "");
		Ok(())
	}
}
// endregion: --- Tests
