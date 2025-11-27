use mlua::IntoLua;
use serde::Serialize;

/// A parsed Markdown inline reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdRef {
	/// URL, file path, or in-document anchor
	pub target: String,
	/// Content inside the brackets
	pub text: Option<String>,
	/// True if prefixed with '!['
	pub inline: bool,
	/// Classification of target
	pub kind: MdRefKind,
}

/// Classification of the reference target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MdRefKind {
	/// "#section"
	Anchor,

	/// "docs/page.md", "image.png", etc.
	File,

	/// "https://example.com"
	Url,
}

impl MdRefKind {
	pub fn from_target(target: &str) -> Self {
		if target.starts_with('#') {
			MdRefKind::Anchor
		} else if target.starts_with("http://") || target.starts_with("https://") || target.starts_with("//") {
			MdRefKind::Url
		} else {
			MdRefKind::File
		}
	}
}

// region:    --- Serde Serializer

impl Serialize for MdRef {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeStruct;
		let mut state = serializer.serialize_struct("MdRef", 5)?;
		state.serialize_field("_type", "MdRef")?;
		state.serialize_field("target", &self.target)?;

		if let Some(text) = &self.text {
			state.serialize_field("text", text)?;
		}
		state.serialize_field("inline", &self.inline)?;
		state.serialize_field("kind", &self.kind)?;

		state.end()
	}
}

impl Serialize for MdRefKind {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let kind_str = match self {
			MdRefKind::Anchor => "Anchor",
			MdRefKind::File => "File",
			MdRefKind::Url => "Url",
		};
		serializer.serialize_str(kind_str)
	}
}

// endregion: --- Serde Serializer

// region:    --- Lua

impl IntoLua for MdRef {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("_type", "MdRef")?;
		table.set("target", self.target)?;
		table.set("text", self.text)?;
		table.set("inline", self.inline)?;
		table.set("kind", self.kind)?;
		Ok(mlua::Value::Table(table))
	}
}

impl IntoLua for MdRefKind {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let kind_str = match self {
			MdRefKind::Anchor => "Anchor",
			MdRefKind::File => "File",
			MdRefKind::Url => "Url",
		};
		kind_str.into_lua(lua)
	}
}

// endregion: --- Lua
