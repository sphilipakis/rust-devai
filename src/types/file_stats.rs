/// Aggregate statistics for a set of files.
/// For `aip.file.stats(globs: string | string[])`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FileStats {
	/// Sum of all file sizes, in bytes.
	pub total_size: u64,
	/// Total number of files.
	pub number_of_files: u64,
	/// Earliest creation time (epoch microseconds).
	pub ctime_first: i64,
	/// Latest creation time (epoch microseconds).
	pub ctime_last: i64,
	/// Earliest modification time (epoch microseconds).
	pub mtime_first: i64,
	/// Latest modification time (epoch microseconds).
	pub mtime_last: i64,
}

// region:    --- Serde Serializer

use serde::{Serialize, Serializer};

impl Serialize for FileStats {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		use serde::ser::SerializeStruct;
		// Max 7 fields (total_size, number_of_files, ctime_first, ctime_last, mtime_first, mtime_last, _type)
		let mut state = serializer.serialize_struct("FileStats", 7)?;

		state.serialize_field("_type", "FileStats")?;
		state.serialize_field("total_size", &self.total_size)?;
		state.serialize_field("number_of_files", &self.number_of_files)?;
		state.serialize_field("ctime_first", &self.ctime_first)?;
		state.serialize_field("ctime_last", &self.ctime_last)?;
		state.serialize_field("mtime_first", &self.mtime_first)?;
		state.serialize_field("mtime_last", &self.mtime_last)?;

		state.end()
	}
}

// endregion: --- Serde Serializer

// region:    --- Lua

use mlua::{IntoLua, Lua};

impl IntoLua for FileStats {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("_type", "FileStats")?;

		table.set("total_size", self.total_size)?;
		table.set("number_of_files", self.number_of_files)?;
		table.set("ctime_first", self.ctime_first)?;
		table.set("ctime_last", self.ctime_last)?;
		table.set("mtime_first", self.mtime_first)?;
		table.set("mtime_last", self.mtime_last)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
