use crate::Result;
use crate::runtime::Runtime;
use mlua::{Lua, Table};

use super::*; // Will pull from text_common, text_split, text_trim via the updated mod.rs

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	// --- Functions from text_common.rs (original, minus moved ones)
	table.set("escape_decode", lua.create_function(escape_decode)?)?;
	table.set("escape_decode_if_needed", lua.create_function(escape_decode_if_needed)?)?;
	table.set("remove_first_line", lua.create_function(remove_first_line)?)?;
	table.set("remove_first_lines", lua.create_function(remove_first_lines)?)?;
	table.set("remove_last_lines", lua.create_function(remove_last_lines)?)?;
	table.set("remove_last_line", lua.create_function(remove_last_line)?)?;
	table.set("trim", lua.create_function(trim)?)?;
	table.set("trim_start", lua.create_function(trim_start)?)?;
	table.set("trim_end", lua.create_function(trim_end)?)?;
	table.set("truncate", lua.create_function(truncate)?)?;
	table.set(
		"replace_markers",
		lua.create_function(replace_markers_with_default_parkers)?,
	)?;
	table.set("extract_line_blocks", lua.create_function(extract_line_blocks)?)?;

	// --- Functions from text_split.rs (via super::*)
	table.set("split_first", lua.create_function(split_first)?)?;
	table.set("split_last", lua.create_function(split_last)?)?;

	// --- Functions from text_trim.rs (ensure functions, via super::*)
	table.set("ensure", lua.create_function(ensure)?)?;
	table.set(
		"ensure_single_ending_newline",
		lua.create_function(ensure_single_ending_newline)?,
	)?;

	Ok(table)
}
