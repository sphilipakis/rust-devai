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
