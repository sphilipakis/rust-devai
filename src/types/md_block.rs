use mlua::IntoLua;
use serde::Serialize;

/// Represents a Markdown block with optional language and content.
#[derive(Debug)]
pub struct MdBlock {
	pub lang: Option<String>,
	pub content: String,
}

impl MdBlock {
	/// Creates a new `MdBlock` with the specified language and content.
	#[allow(unused)]
	pub fn new(lang: Option<String>, content: impl Into<String>) -> Self {
		MdBlock {
			lang,
			content: content.into(),
		}
	}
}

// region:    --- Serde Serializer

impl Serialize for MdBlock {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeStruct;
		let mut state = serializer.serialize_struct("MdBlock", 3)?;
		state.serialize_field("_type", "MdBlock")?;

		if let Some(lang) = &self.lang {
			state.serialize_field("lang", lang)?;
		}
		state.serialize_field("content", &self.content)?;

		state.end()
	}
}

// endregion: --- Serde Serializer

// region:    --- Lua

impl IntoLua for MdBlock {
	/// Converts the `MdBlock` instance into a Lua Value
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("_type", "MdBlock")?;

		table.set("lang", self.lang)?;
		table.set("content", self.content)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
