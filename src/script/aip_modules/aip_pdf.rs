use crate::runtime::Runtime;
use crate::support::pdf;
use crate::{Error, Result};
use mlua::{Lua, Table};
use simple_fs::SPath;

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let page_count_fn = lua.create_function(move |_lua, path: String| page_count(path))?;

	table.set("page_count", page_count_fn)?;

	Ok(table)
}

/// Lua: `aip.pdf.page_count(path): number`
fn page_count(path: String) -> mlua::Result<usize> {
	let spath = SPath::from_std_path(&path).map_err(|err| Error::custom(format!("aip.pdf.page_count failed. {err}")))?;

	let doc = pdf::load_pdf_doc(&spath).map_err(|err| Error::custom(format!("aip.pdf.page_count failed. {err}")))?;

	Ok(pdf::page_count(&doc))
}
