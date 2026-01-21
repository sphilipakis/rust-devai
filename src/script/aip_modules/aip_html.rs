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
//! - `aip.html.select(html_content: string, selectors: string | string[]) -> Elem[]`
//! - `aip.html.to_md(html_content: string) -> string`

use crate::runtime::Runtime;
use crate::script::support::into_vec_of_strings;
use crate::support::W;
use crate::support::text::trim_if_needed;
use crate::{Result, support};
use html_helpers::Elem;
use mlua::{IntoLua, Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let slim_fn = lua.create_function(html_slim)?;
	table.set("slim", slim_fn.clone())?;

	let select_fn = lua.create_function(move |lua, (html_content, selectors): (String, Value)| {
		html_select(lua, html_content, selectors)
	})?;
	table.set("select", select_fn)?;

	let to_md_fn = lua.create_function(html_to_md)?;
	table.set("to_md", to_md_fn)?;

	// deprecated (TODO: need to send a deprecation notice once we have the deprecation)
	table.set("prune_to_content", slim_fn)?;

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
		.map_err(|err| mlua::Error::RuntimeError(format!("Failed to prune HTML content: {err}")))
}

fn html_select(lua: &Lua, html_content: String, selectors: Value) -> mlua::Result<Value> {
	// if selectors, nil, then empty table
	if selectors.is_nil() {
		let seq = lua.create_sequence_from(Vec::<String>::new())?;
		return Ok(Value::Table(seq));
	}

	// extract as selectors
	let selectors = into_vec_of_strings(selectors, "aip.html.select")?;

	let els = html_helpers::select(&html_content, &selectors)
		.map_err(|err| crate::Error::custom(format!("Cannot apply selector '{selectors:?}'.\nCause: {err}")))?;

	let els: Vec<mlua::Value> = els
		.into_iter()
		.map(|el| W(el).into_lua(lua))
		.collect::<mlua::Result<Vec<_>>>()
		.map_err(|err| {
			crate::Error::custom(format!("aip.html.select cannot make elem into Lua object. Cause {err}"))
		})?;

	let seq = lua.create_sequence_from(els)?;

	Ok(Value::Table(seq))
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
		.map_err(|err| mlua::Error::RuntimeError(format!("Failed to convert HTML to Markdown: {err}")))
}

// region:    --- Froms

impl IntoLua for W<Elem> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let el = self.0;
		let table = lua.create_table()?;
		table.set("tag", el.tag)?;

		// Only set the attribute if present and not empty
		if let Some(attrs) = el.attrs {
			table.set("attrs", attrs)?;
		}

		table.set("text", el.text.map(trim_if_needed))?;
		table.set("inner_html", el.inner_html.map(trim_if_needed))?;

		Ok(Value::Table(table))
	}
}

// endregion: --- Froms

// region:    --- Tests
#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_html;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_lua_html_slim_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_html::init_module, "html").await?;
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
		let cleaned_html = res.as_str().ok_or("Should have res")?;
		assert!(!cleaned_html.contains("<script>"));
		assert!(!cleaned_html.contains("<style>"));
		assert!(!cleaned_html.contains("<!-- comment -->"));
		assert!(cleaned_html.contains(r#"<div class="content">Hello World</div>"#));
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_html_select_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_html::init_module, "html").await?;
		let fx_script = r#"
local html_content = [[
			<div>First div<div>
			<li class="me">Bullet One </li>
			<section>
				<p>Some text</p>
			  <DIV class="me other " TITLE = " Some Title" > Div <strong>Two </strong></DIV>
			</section>
]]
return aip.html.select(html_content, ".me")
		"#;

		// -- Exec
		let res = eval_lua(&lua, fx_script)?;

		// -- Check
		let res = res.as_array().ok_or("Should be array")?;
		assert_eq!(res.len(), 2);
		// first one (<li>)
		let el = res.first().ok_or("Should have at least one")?;
		assert_eq!(el.x_get_str("tag")?, "li");
		assert_eq!(el.x_get_str("/attrs/class")?, "me");
		assert_eq!(el.x_get_str("text")?, "Bullet One");
		assert_eq!(el.x_get_str("inner_html")?, "Bullet One");
		// second one (<div>)
		let el = res.get(1).ok_or("Should have at least two")?;
		assert_eq!(el.x_get_str("tag")?, "div");
		assert_eq!(el.x_get_str("/attrs/class")?, "me other ");
		assert_eq!(el.x_get_str("/attrs/title")?, " Some Title");
		assert_eq!(el.x_get_str("text")?, "Div Two");
		assert_eq!(el.x_get_str("inner_html")?, "Div <strong>Two </strong>");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_html_to_md_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_html::init_module, "html").await?;
		let fx_script = r#"
local html_content = "<h1>Title</h1><p>Some <strong>bold</strong> text.</p><ul><li>Item 1</li><li>Item 2</li></ul>"
return aip.html.to_md(html_content)
        "#;

		// -- Exec
		let res = eval_lua(&lua, fx_script)?;

		// -- Check
		let md_content = res.as_str().ok_or("Result should be string")?;
		let expected_md = "# Title\n\nSome **bold** text.\n\n- Item 1\n- Item 2";
		assert_eq!(md_content, expected_md);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_html_to_md_empty_input() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_html::init_module, "html").await?;
		let fx_script = r#"
return aip.html.to_md("")
        "#;

		// -- Exec
		let res = eval_lua(&lua, fx_script)?;

		// -- Check
		let md_content = res.as_str().ok_or("Should have res")?;
		assert_eq!(md_content, "");
		Ok(())
	}
}
// endregion: --- Tests
