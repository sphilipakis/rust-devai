use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use mlua::{Lua, Value};
use simple_fs::{SPath, csv_row_spans as fs_csv_row_spans, line_spans as fs_line_spans, read_span as fs_read_span};

// region:    --- Lua Spans

pub(super) fn file_line_spans(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let rel_path = SPath::new(path);
	let full_path = runtime
		.dir_context()
		.resolve_path(runtime.session(), rel_path, PathResolver::WksDir, None)?;

	let spans = fs_line_spans(&full_path).map_err(Error::from)?;
	let table = spans_to_lua_table(lua, &spans)?;
	Ok(Value::Table(table))
}

pub(super) fn file_csv_row_spans(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let rel_path = SPath::new(path);
	let full_path = runtime
		.dir_context()
		.resolve_path(runtime.session(), rel_path, PathResolver::WksDir, None)?;

	let spans = fs_csv_row_spans(&full_path).map_err(Error::from)?;
	let table = spans_to_lua_table(lua, &spans)?;
	Ok(Value::Table(table))
}

pub(super) fn file_read_span(
	_lua: &Lua,
	runtime: &Runtime,
	path: String,
	start: i64,
	end: i64,
) -> mlua::Result<String> {
	if start < 0 || end < 0 {
		return Err(Error::custom("read_span expects non-negative start/end offsets").into());
	}
	if end < start {
		return Err(Error::custom("read_span expects end >= start").into());
	}

	let rel_path = SPath::new(path);
	let full_path = runtime
		.dir_context()
		.resolve_path(runtime.session(), rel_path, PathResolver::WksDir, None)?;

	let text = fs_read_span(&full_path, start as usize, end as usize).map_err(Error::from)?;
	Ok(text)
}

// endregion: --- Lua Spans

// region:    --- Support

fn spans_to_lua_table(lua: &Lua, spans: &[(usize, usize)]) -> mlua::Result<mlua::Table> {
	let out = lua.create_table()?;
	for (i, (start, end)) in spans.iter().enumerate() {
		let row = lua.create_table()?;
		row.set("start", *start as i64)?;
		row.set("end", *end as i64)?;
		out.set(i + 1, row)?;
	}
	Ok(out)
}

// endregion: --- Support
