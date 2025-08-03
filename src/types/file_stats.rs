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

// region:    --- Lua

use mlua::{IntoLua, Lua};

impl IntoLua for FileStats {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
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
