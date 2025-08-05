use crate::types::MdHeading;
use mlua::IntoLua;
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct MdSection {
	pub content: String,
	pub heading: Option<MdHeading>,
}

/// Constructors
/// For now, constructe by
#[allow(unused)]
impl MdSection {
	pub fn from_content(content: impl Into<String>) -> Self {
		Self {
			content: content.into(),
			heading: None,
		}
	}
	pub fn new(content: String, heading: impl Into<Option<MdHeading>>) -> Self {
		Self {
			content,
			heading: heading.into(),
		}
	}
}

/// Getters
impl MdSection {
	/// The content of the section
	/// NOTE: The eventual end newline for this section in the markdown is not included in the content
	#[allow(unused)]
	pub fn content(&self) -> &str {
		&self.content
	}

	#[allow(unused)]
	pub fn heading(&self) -> Option<&MdHeading> {
		self.heading.as_ref()
	}

	// Convenient function that returnes the heading content
	// - "" empty string if not present
	// - "{content}\n" if present (adding the new line)
	pub fn heading_raw(&self) -> String {
		self.heading
			.as_ref()
			.map_or_else(|| "".to_string(), |h| h.content().to_string() + "\n")
	}
}

// region:    --- Serde Serializer

impl Serialize for MdSection {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		use serde::ser::SerializeStruct;
		// Max 3 fields (content, heading, _type)
		let mut state = serializer.serialize_struct("MdSection", 3)?;
		state.serialize_field("_type", "MdSection")?;

		state.serialize_field("content", &self.content)?;
		if let Some(heading) = &self.heading {
			state.serialize_field("heading", heading)?;
		}

		state.end()
	}
}

// endregion: --- Serde Serializer

// region:    --- Lua

impl IntoLua for MdSection {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("_type", "MdSection")?;

		let heading_raw = self.heading_raw();

		table.set("content", self.content)?;
		table.set("heading_raw", heading_raw)?;
		if let Some(heading) = self.heading {
			table.set("heading_content", heading.content())?;
			table.set("heading_level", heading.level())?;
			table.set("heading_name", heading.name())?;
		} else {
			table.set("heading_content", "")?;
			table.set("heading_level", 0)?;
			table.set("heading_name", "heading.name()")?;
		}
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
