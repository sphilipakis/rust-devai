use crate::Result;
use crate::runtime::Runtime;
use crate::script::aip_modules::aip_file::file_common::{
	EnsureExistsOptions, file_append, file_ensure_exists, file_first, file_list, file_list_load, file_load, file_save,
};
use crate::script::aip_modules::aip_file::file_html::file_save_html_to_md;
use crate::script::aip_modules::aip_file::file_json::{
	file_append_json_line, file_append_json_lines, file_load_json, file_load_ndjson,
};
use crate::script::aip_modules::aip_file::file_md::{file_load_md_sections, file_load_md_split_first};
use mlua::{Lua, Table, Value};

// endregion: --- Modules

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	// -- load
	let rt = runtime.clone();
	let file_load_fn =
		lua.create_function(move |lua, (path, options): (String, Option<Value>)| file_load(lua, &rt, path, options))?;

	// -- save
	let rt = runtime.clone();
	let file_save_fn =
		lua.create_function(move |lua, (path, content): (String, String)| file_save(lua, &rt, path, content))?;

	// -- append
	let rt = runtime.clone();
	let file_append_fn =
		lua.create_function(move |lua, (path, content): (String, String)| file_append(lua, &rt, path, content))?;

	// -- ensure_exists
	let rt = runtime.clone();
	let file_ensure_exists_fn = lua.create_function(
		move |lua, (path, content, options): (String, Option<String>, Option<EnsureExistsOptions>)| {
			file_ensure_exists(lua, &rt, path, content, options)
		},
	)?;

	// -- list
	let rt = runtime.clone();
	let file_list_fn =
		lua.create_function(move |lua, (globs, options): (Value, Option<Value>)| file_list(lua, &rt, globs, options))?;

	// -- list_load
	let rt = runtime.clone();
	let file_list_load_fn = lua.create_function(move |lua, (globs, options): (Value, Option<Value>)| {
		file_list_load(lua, &rt, globs, options)
	})?;

	// -- first
	let rt = runtime.clone();
	let file_first_fn =
		lua.create_function(move |lua, (globs, options): (Value, Option<Value>)| file_first(lua, &rt, globs, options))?;

	// -- load_json
	let rt = runtime.clone();
	let file_load_json_fn = lua.create_function(move |lua, (path,): (String,)| file_load_json(lua, &rt, path))?;

	// -- load_ndjson
	let rt = runtime.clone();
	let file_load_ndjson_fn = lua.create_function(move |lua, (path,): (String,)| file_load_ndjson(lua, &rt, path))?;

	// -- append_json_line
	let rt = runtime.clone();
	let file_append_json_line_fn =
		lua.create_function(move |lua, (path, data): (String, Value)| file_append_json_line(lua, &rt, path, data))?;

	// -- append_json_lines
	let rt = runtime.clone();
	let file_append_json_lines_fn =
		lua.create_function(move |lua, (path, data): (String, Value)| file_append_json_lines(lua, &rt, path, data))?;

	// -- load_md_sections
	let rt = runtime.clone();
	let file_load_md_sections_fn = lua.create_function(move |lua, (path, headings): (String, Option<Value>)| {
		file_load_md_sections(lua, &rt, path, headings)
	})?;

	// -- load_md_split_first
	let rt = runtime.clone();
	let file_load_md_split_first_fn =
		lua.create_function(move |lua, (path,): (String,)| file_load_md_split_first(lua, &rt, path))?;

	// -- save_html_to_md
	let rt = runtime.clone();
	let file_save_html_to_md_fn = lua.create_function(move |lua, (html_path, dest_options): (String, Value)| {
		file_save_html_to_md(lua, &rt, html_path, dest_options)
	})?;

	// -- Add all functions to the module
	table.set("load", file_load_fn)?;
	table.set("save", file_save_fn)?;
	table.set("append", file_append_fn)?;
	table.set("ensure_exists", file_ensure_exists_fn)?;
	table.set("list", file_list_fn)?;
	table.set("list_load", file_list_load_fn)?;
	table.set("first", file_first_fn)?;
	table.set("load_json", file_load_json_fn)?;
	table.set("load_ndjson", file_load_ndjson_fn)?;
	table.set("append_json_line", file_append_json_line_fn)?;
	table.set("append_json_lines", file_append_json_lines_fn)?;
	table.set("load_md_sections", file_load_md_sections_fn)?;
	table.set("load_md_split_first", file_load_md_split_first_fn)?;
	table.set("save_html_to_md", file_save_html_to_md_fn)?;

	Ok(table)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{
		assert_contains, assert_not_contains, clean_sanbox_01_tmp_file, create_sanbox_01_tmp_file,
		gen_sandbox_01_temp_file_path, resolve_sandbox_01_path, run_reflective_agent,
	};
	use simple_fs::{SPath, read_to_string as sfs_read_to_string};
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
		let fx_html_rel_path = create_sanbox_01_tmp_file(
			"test_script_aip_file_save_html_to_md_simple_ok-input.html",
			fx_html_content,
		)?;
		// let fx_html_rel_path = resolve_sandbox_01_path(&fx_html_path);

		let md_rel_path = fx_html_rel_path.new_sibling("test_script_aip_file_save_html_to_md_simple_ok-output.md");

		// -- Exec
		let lua_code = format!(
			r#"return aip.file.save_html_to_md("{}", "{}")"#,
			fx_html_rel_path, md_rel_path
		);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		assert_eq!(res.x_get_str("path")?, md_rel_path.as_str());
		assert_eq!(res.x_get_str("ext")?, "md");
		assert!(res.x_get_i64("size")? > 0);

		let md_full_path = resolve_sandbox_01_path(&md_rel_path);
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
		clean_sanbox_01_tmp_file(fx_html_rel_path)?;
		clean_sanbox_01_tmp_file(md_rel_path)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_md_with_options_table() -> Result<()> {
		// -- Setup & Fixtures
		let fx_html_content = "<h1>Hello</h1>";
		let fx_html_rel_path = create_sanbox_01_tmp_file("test_save_html_opts_table-input.html", fx_html_content)?;
		let fx_html_stem = fx_html_rel_path.stem();

		let fx_options_lua = r#"{ base_dir = ".tmp/output", suffix = "_converted" }"#;
		let expected_md_rel_path = SPath::new(format!(".tmp/output/{fx_html_stem}_converted.md"));

		// -- Exec
		let lua_code = format!(
			r#"return aip.file.save_html_to_md("{}", {})"#,
			fx_html_rel_path, fx_options_lua
		);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		assert_eq!(res.x_get_str("path")?, expected_md_rel_path.as_str());
		let md_full_path = resolve_sandbox_01_path(&expected_md_rel_path);
		assert!(md_full_path.exists(), "Markdown file should be created");
		let md_content = sfs_read_to_string(&md_full_path)?;
		assert_contains(&md_content, "# Hello");

		// -- Cleanup
		clean_sanbox_01_tmp_file(fx_html_rel_path)?;
		clean_sanbox_01_tmp_file(md_full_path)?;
		// Also clean the .tmp/output directory if it's empty, or specific file
		let output_dir = resolve_sandbox_01_path(&SPath::new(".tmp/output"));
		if output_dir.is_dir() {
			let _ = std::fs::remove_dir(output_dir); // ignore error if not empty
		}

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

		let Err(err) = run_reflective_agent(&lua_code, None).await else {
			panic!("Should have returned an error")
		};

		let msg = err.to_string();
		assert_contains(&msg, "Failed to read HTML file ");

		Ok(())
	}
}

// endregion: --- Tests
