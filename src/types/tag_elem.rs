#![allow(unused)] // for now, as not wired yet

use mlua::IntoLua;
use serde::Serialize;
use std::collections::HashMap;

/// Represents a block defined by start and end tags, like `<TAG>content</TAG>`.
#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct TagElem {
	pub tag: String, // might want to set this a Arc<str>

	pub attrs: Option<HashMap<String, String>>,

	pub content: String,
}

impl TagElem {
	/// Creates a new `TagElem` with the specified name, optional attributes, and content.
	pub fn new(name: impl Into<String>, attrs: Option<HashMap<String, String>>, content: impl Into<String>) -> Self {
		TagElem {
			tag: name.into(),
			attrs,
			content: content.into(),
		}
	}
}

// region:    --- Lua

impl IntoLua for TagElem {
	/// Converts the `TagElem` instance into a Lua Value
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("tag", self.tag)?;
		table.set("attrs", self.attrs)?; // Note: Lua might need handling for Option<HashMap>
		table.set("content", self.content)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
