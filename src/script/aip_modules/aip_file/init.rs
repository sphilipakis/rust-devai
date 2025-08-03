use crate::Result;
use crate::runtime::Runtime;
use crate::script::aip_modules::aip_file::file_change::file_save_changes;
use crate::script::aip_modules::aip_file::file_read::{
	file_exists, file_first, file_info, file_list, file_list_load, file_load, file_stats,
};
use crate::script::aip_modules::aip_file::file_write::{
	file_append, file_ensure_exists, file_save, EnsureExistsOptions,
};
use crate::script::aip_modules::aip_file::file_hash::{
	file_hash_blake3, file_hash_blake3_b58u, file_hash_blake3_b64, file_hash_blake3_b64u, file_hash_sha256,
	file_hash_sha256_b58u, file_hash_sha256_b64, file_hash_sha256_b64u, file_hash_sha512, file_hash_sha512_b58u,
	file_hash_sha512_b64, file_hash_sha512_b64u,
};
use crate::script::aip_modules::aip_file::file_html::{file_save_html_to_md, file_save_html_to_slim};
use crate::script::aip_modules::aip_file::file_json::{
	file_append_json_line, file_append_json_lines, file_load_json, file_load_ndjson,
};
use crate::script::aip_modules::aip_file::file_md::{file_load_md_sections, file_load_md_split_first};
use mlua::{Lua, Table, Value};

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

	// -- exists
	let rt = runtime.clone();
	let file_exists_fn = lua.create_function(move |lua, path: String| file_exists(lua, &rt, path))?;

	// -- info
	let rt = runtime.clone();
	let file_info_fn = lua.create_function(move |lua, path: Value| file_info(lua, &rt, path))?;

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

	// -- stats
	let rt = runtime.clone();
	let file_stats_fn =
		lua.create_function(move |lua, (globs, options): (Value, Option<Value>)| file_stats(lua, &rt, globs, options))?;

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

	// -- save_html_to_slim
	let rt = runtime.clone();
	let file_save_html_to_slim_fn = lua.create_function(move |lua, (html_path, dest_options): (String, Value)| {
		file_save_html_to_slim(lua, &rt, html_path, dest_options)
	})?;

	// -- save_chages
	let rt = runtime.clone();
	let file_save_changes_fn =
		lua.create_function(move |lua, (path, changes): (String, String)| file_save_changes(lua, &rt, path, changes))?;

	// -- File Hash Functions
	let rt = runtime.clone();
	let file_hash_sha256_fn = lua.create_function(move |lua, path: String| file_hash_sha256(lua, &rt, path))?;
	let rt = runtime.clone();
	let file_hash_sha256_b64_fn = lua.create_function(move |lua, path: String| file_hash_sha256_b64(lua, &rt, path))?;
	let rt = runtime.clone();
	let file_hash_sha256_b64u_fn =
		lua.create_function(move |lua, path: String| file_hash_sha256_b64u(lua, &rt, path))?;
	let rt = runtime.clone();
	let file_hash_sha256_b58u_fn =
		lua.create_function(move |lua, path: String| file_hash_sha256_b58u(lua, &rt, path))?;

	let rt = runtime.clone();
	let file_hash_sha512_fn = lua.create_function(move |lua, path: String| file_hash_sha512(lua, &rt, path))?;
	let rt = runtime.clone();
	let file_hash_sha512_b64_fn = lua.create_function(move |lua, path: String| file_hash_sha512_b64(lua, &rt, path))?;
	let rt = runtime.clone();
	let file_hash_sha512_b64u_fn =
		lua.create_function(move |lua, path: String| file_hash_sha512_b64u(lua, &rt, path))?;
	let rt = runtime.clone();
	let file_hash_sha512_b58u_fn =
		lua.create_function(move |lua, path: String| file_hash_sha512_b58u(lua, &rt, path))?;

	let rt = runtime.clone();
	let file_hash_blake3_fn = lua.create_function(move |lua, path: String| file_hash_blake3(lua, &rt, path))?;
	let rt = runtime.clone();
	let file_hash_blake3_b64_fn = lua.create_function(move |lua, path: String| file_hash_blake3_b64(lua, &rt, path))?;
	let rt = runtime.clone();
	let file_hash_blake3_b64u_fn =
		lua.create_function(move |lua, path: String| file_hash_blake3_b64u(lua, &rt, path))?;
	let rt = runtime.clone();
	let file_hash_blake3_b58u_fn =
		lua.create_function(move |lua, path: String| file_hash_blake3_b58u(lua, &rt, path))?;

	// -- Add all functions to the module
	table.set("load", file_load_fn)?;
	table.set("save", file_save_fn)?;
	table.set("append", file_append_fn)?;
	table.set("ensure_exists", file_ensure_exists_fn)?;
	table.set("exists", file_exists_fn)?;
	table.set("info", file_info_fn)?;
	table.set("list", file_list_fn)?;
	table.set("list_load", file_list_load_fn)?;
	table.set("first", file_first_fn)?;
	table.set("stats", file_stats_fn)?;
	table.set("load_json", file_load_json_fn)?;
	table.set("load_ndjson", file_load_ndjson_fn)?;
	table.set("append_json_line", file_append_json_line_fn)?;
	table.set("append_json_lines", file_append_json_lines_fn)?;
	table.set("load_md_sections", file_load_md_sections_fn)?;
	table.set("load_md_split_first", file_load_md_split_first_fn)?;
	table.set("save_html_to_md", file_save_html_to_md_fn)?;
	table.set("save_html_to_slim", file_save_html_to_slim_fn)?;
	table.set("save_changes", file_save_changes_fn)?;

	// -- Add file hash functions
	table.set("hash_sha256", file_hash_sha256_fn)?;
	table.set("hash_sha256_b64", file_hash_sha256_b64_fn)?;
	table.set("hash_sha256_b64u", file_hash_sha256_b64u_fn)?;
	table.set("hash_sha256_b58u", file_hash_sha256_b58u_fn)?;
	table.set("hash_sha512", file_hash_sha512_fn)?;
	table.set("hash_sha512_b64", file_hash_sha512_b64_fn)?;
	table.set("hash_sha512_b64u", file_hash_sha512_b64u_fn)?;
	table.set("hash_sha512_b58u", file_hash_sha512_b58u_fn)?;
	table.set("hash_blake3", file_hash_blake3_fn)?;
	table.set("hash_blake3_b64", file_hash_blake3_b64_fn)?;
	table.set("hash_blake3_b64u", file_hash_blake3_b64u_fn)?;
	table.set("hash_blake3_b58u", file_hash_blake3_b58u_fn)?;

	Ok(table)
}
