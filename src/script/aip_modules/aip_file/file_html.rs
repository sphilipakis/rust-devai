use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;

use crate::types::FileMeta;
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

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{
		assert_contains, clean_sanbox_01_tmp_file, create_sanbox_01_tmp_file, gen_sandbox_01_temp_file_path,
		resolve_sandbox_01_path, run_reflective_agent,
	};
	use simple_fs::read_to_string as sfs_read_to_string;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_md_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_html_content = r#"
<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
    <h1>Main Title</h1>
    <p>This is a paragraph with <strong>strong</strong> text and <em>emphasized</em> text.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
    </ul>
    <a href="https://example.com">A Link</a>
</body>
</html>"#;
		let fx_html_path = create_sanbox_01_tmp_file(
			"test_script_aip_file_save_html_to_md_simple_ok-input.html",
			fx_html_content,
		)?;

		let md_path = fx_html_path.new_sibling("test_script_aip_file_save_html_to_md_simple_ok-input.md");

		// -- Exec
		let lua_code = format!(r#"return aip.file.save_html_to_md("{}", "{}")"#, fx_html_path, md_path);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		// Check FileMeta result
		assert_eq!(res.x_get_str("path")?, md_path.as_str());
		assert_eq!(res.x_get_str("ext")?, "md");
		assert!(res.x_get_i64("size")? > 0);

		// Check MD content
		let md_full_path = resolve_sandbox_01_path(&md_path);
		let md_content = sfs_read_to_string(&md_full_path)?;
		assert_contains(&md_content, "# Main Title");
		assert_contains(
			&md_content,
			"This is a paragraph with **strong** text and _emphasized_ text.",
		);
		assert_contains(&md_content, "-   Item 1");
		assert_contains(&md_content, "-   Item 2");
		assert_contains(&md_content, "[A Link](https://example.com)");

		// -- Cleanup
		clean_sanbox_01_tmp_file(md_full_path)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_md_html_not_found() -> Result<()> {
		let fx_src_path = gen_sandbox_01_temp_file_path("test_script_aip_file_save_html_to_md_html_not_found.html");
		let fx_dst_path = gen_sandbox_01_temp_file_path("test_script_aip_file_save_html_to_md_html_not_found.md");

		// -- Exec
		let lua_code = format!(
			r#"return aip.file.save_html_to_md("{}", "{}")"#,
			fx_src_path, fx_dst_path
		);

		let Err(res) = run_reflective_agent(&lua_code, None).await else {
			panic!("Should have returned a error")
		};

		// -- Check
		let res = res.to_string();
		assert_contains(&res, "Failed to read HTML file ");

		Ok(())
	}
}

// endregion: --- Tests
