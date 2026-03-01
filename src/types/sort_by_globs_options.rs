use crate::script::LuaValueExt;
use crate::support::W;
use mlua::{FromLua, Lua, Value};
use simple_fs::{NoMatchPosition, SortByGlobsOptions};

impl FromLua for W<SortByGlobsOptions> {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(W(SortByGlobsOptions::default())),
			Value::Boolean(end_weighted) => Ok(W(SortByGlobsOptions {
				end_weighted,
				no_match_position: NoMatchPosition::End,
			})),
			Value::String(s) => {
				let s = s
					.to_str()
					.map_err(|e| mlua::Error::runtime(format!("aip.path.sort_by_globs options string error: {e}")))?;
				let s = s.to_string();
				let no_match_position = match s.as_str() {
					"start" => NoMatchPosition::Start,
					"end" => NoMatchPosition::End,
					other => {
						return Err(mlua::Error::runtime(format!(
							"aip.path.sort_by_globs options string must be 'start' or 'end', got '{other}'"
						)));
					}
				};
				Ok(W(SortByGlobsOptions {
					end_weighted: false,
					no_match_position,
				}))
			}
			Value::Table(tbl) => {
				let end_weighted = tbl
					.get::<Value>("end_weighted")
					.ok()
					.and_then(|v| v.as_boolean())
					.unwrap_or(false);
				let no_match_position = tbl
					.get::<Value>("no_match_position")
					.ok()
					.and_then(|v| v.x_to_string())
					.map(|s| match s.as_ref() {
						"start" => NoMatchPosition::Start,
						_ => NoMatchPosition::End,
					})
					.unwrap_or(NoMatchPosition::End);
				Ok(W(SortByGlobsOptions {
					end_weighted,
					no_match_position,
				}))
			}
			_ => Err(mlua::Error::runtime(
				"aip.path.sort_by_globs options must be nil, boolean, 'start'/'end' string, or a table",
			)),
		}
	}
}
