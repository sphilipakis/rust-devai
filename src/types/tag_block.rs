#![allow(unused)] // for now, as not wired yet

use mlua::IntoLua;
use serde::Serialize;
use std::collections::HashMap;

/// Represents a block defined by start and end tags, like `<TAG>content</TAG>`.
#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct TagBlock {
	pub name: String, // might want to set this a Arc<str>

	// For now, parsing is skipped, will be added later.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attrs: Option<HashMap<String, String>>,

	pub content: String,
}

impl TagBlock {
	/// Creates a new `TagBlock` with the specified name, optional attributes, and content.
	pub fn new(name: impl Into<String>, attrs: Option<HashMap<String, String>>, content: impl Into<String>) -> Self {
		TagBlock {
			name: name.into(),
			attrs,
			content: content.into(),
		}
	}
}

// region:    --- Lua

impl IntoLua for TagBlock {
	/// Converts the `TagBlock` instance into a Lua Value
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("name", self.name)?;
		table.set("attrs", self.attrs)?; // Note: Lua might need handling for Option<HashMap>
		table.set("content", self.content)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
