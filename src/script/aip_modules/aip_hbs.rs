//! Defines the `aip.hbs` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.hbs` module exposes functions to render Handlebars templates with data.
//!
//! ### Functions
//!
//! - `aip.hbs.render(content: string, data: any): string`

use crate::Result;
use crate::runtime::Runtime;
use mlua::{Lua, Table, Value};

/// Initializes the `hbs` Lua module.
///
/// Registers the `render` function in the module table.
pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;
	table.set("render", lua.create_function(render)?)?;
	Ok(table)
}

/// ## Lua Documentation
///
/// Renders a Handlebars template with the given content and data.
///
/// ```lua
/// -- API Signature
/// aip.hbs.render(content: string, data: any): string
/// ```
///
/// The `render` function takes a Handlebars template string and a Lua value as input,
/// converts the Lua value to a `serde_json::Value`, and renders the template.
///
/// ### Arguments
///
/// - `content: string`: The Handlebars template as a string.
/// - `data: any`: The data as a Lua value (table, number, string, boolean, nil). Note that function types or userdata are not supported.
///
/// ### Returns
///
/// `string`: The rendered template as a string.
///
/// ### Example
///
/// ```lua
/// -- Simple example
/// local rendered_content = aip.hbs.render("Hello, {{name}}!", {name = "World"})
/// print(rendered_content) -- Output: Hello, World!
///
/// -- Example with a list
/// local data = {
///     name  = "Jen Donavan",
///     todos = {"Bug Triage AIPACK", "Fix Windows Support"}
/// }
///
/// local template = [[
/// Hello {{name}},
///
/// Your tasks today:
///
/// {{#each todos}}
/// - {{this}}
/// {{/each}}
///
/// Have a good day (after you completed this tasks)
/// ]]
///
/// local content = aip.hbs.render(template, data)
/// print(content)
/// ```
///
/// ### Error
///
/// Returns an error if:
///
/// - The Lua value cannot be converted to a `serde_json::Value`.
/// - The Handlebars template rendering fails.
///
/// ```ts
/// {
///   error : string // Error message
/// }
/// ```
fn render(_lua: &Lua, (content, data): (String, Value)) -> mlua::Result<String> {
	let data_serde = serde_json::to_value(&data)
		.map_err(|err| crate::Error::custom(format!("Fail to convert lua value to serde. Cause: {err}")))?;
	let rendered = crate::support::hbs::hbs_render(&content, &data_serde).map_err(mlua::Error::external)?;
	Ok(rendered)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules::aip_hbs;

	#[tokio::test]
	async fn test_lua_hbs_render_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hbs::init_module, "hbs").await?;

		// -- Exec
		let lua_code = r#"
            local result = aip.hbs.render("Hello, {{name}}!", {name = "World"})
            return result
		"#;
		let res = eval_lua(&lua, lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, "Hello, World!");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hbs_render_obj() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hbs::init_module, "hbs").await?;

		// -- Exec
		let lua_code = r#"
            local result = aip.hbs.render("ID: {{id}}, Nested: {{nested.value}}", {id = 42, nested = {value = "test"}})
            return result
		"#;
		let res = eval_lua(&lua, lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, "ID: 42, Nested: test");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hbs_render_list() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hbs::init_module, "hbs").await?;

		// -- Exec
		let lua_code = r#"
local data = {
    name  = "Jen Donavan",
    todos = {"Bug Triage AIPACK", "Fix Windows Support"}
}

local template = [[
Hello {{name}},

Your tasks today:

{{#each todos}}
- {{this}}
{{/each}}

Have a good day (after you completed this tasks)
]]

local content = aip.hbs.render(template, data)

return content
		"#;

		// -- Check
		let res = eval_lua(&lua, lua_code)?;

		// -- Test
		let content = res.as_str().ok_or("Should have returned a string")?;
		assert_contains(content, "Hello Jen Donavan");
		assert_contains(content, "- Bug Triage AIPACK");
		assert_contains(content, "- Fix Windows Support");

		Ok(())
	}
}

// endregion: --- Tests
