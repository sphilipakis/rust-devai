use crate::script::serde_value_to_lua_value;
use derive_more::From;
use derive_more::derive::Display;
use mlua::IntoLua;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, From, Display, Clone)]
#[display("{_0:?}")]
pub struct YamlDocs(Vec<Value>);

impl YamlDocs {
	pub fn new(docs: Vec<Value>) -> Self {
		Self(docs)
	}

	pub fn into_first(self) -> Option<Value> {
		self.0.into_iter().next()
	}
}

// region:    --- Lua

impl IntoLua for YamlDocs {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let values = self.0;
		let table = lua.create_table()?;
		for (i, value) in values.into_iter().enumerate() {
			let lua_value = serde_value_to_lua_value(lua, value)?;
			table.set(i + 1, lua_value)?;
		}
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua

// region:    --- Iterators

impl IntoIterator for YamlDocs {
	type Item = Value;
	type IntoIter = std::vec::IntoIter<Value>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a> IntoIterator for &'a YamlDocs {
	type Item = &'a Value;
	type IntoIter = std::slice::Iter<'a, Value>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

// endregion: --- Iterators
