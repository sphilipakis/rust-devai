use crate::dir_context::DirContext;
use crate::{Error, Result};
use mlua::{IntoLua, Lua};
use serde::Serialize;
use simple_fs::SPath;
use std::fs::read_to_string;

/// FileRecord contains the metadata information about the file (name, ext, etc.) as well as the content.
#[derive(Serialize)]
pub struct FileRecord {
	/// The path, might and will probably be relative
	pub path: String,
	/// The dir/parent path of this file from path (will be empty if no parent of the rel path)
	pub dir: String,
	/// The name of the file with extension e.g., `main.rs`
	pub name: String,
	/// Stem
	pub stem: String,
	/// Empty if there is no extension
	pub ext: String,
	/// The full text content of the file
	pub content: String,

	pub ctime: i64,
	pub mtime: i64,
	pub size: i64,
}

/// Constructors
impl FileRecord {
	pub fn load_from_full_path(dir_context: &DirContext, full_path: &SPath, rel_path: SPath) -> Result<Self> {
		let rel_path = dir_context.maybe_home_path_into_tilde(rel_path);
		let content = read_to_string(full_path).map_err(|err| Error::cc(format!("Fail to read {full_path}"), err))?;
		let dir = rel_path.parent().map(|p| p.to_string()).unwrap_or_default();
		let meta = full_path.meta()?;

		Ok(FileRecord {
			path: rel_path.to_string(),
			dir,
			name: rel_path.name().to_string(),
			stem: rel_path.stem().to_string(),
			ext: rel_path.ext().to_string(),
			content,
			ctime: meta.created_epoch_us,
			mtime: meta.modified_epoch_us,
			size: meta.size as i64,
		})
	}
}

// region:    --- Lua

impl IntoLua for FileRecord {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;

		table.set("path", self.path)?;
		table.set("dir", self.dir)?;
		table.set("name", self.name)?;
		table.set("stem", self.stem)?;
		table.set("ext", self.ext)?;

		table.set("ctime", self.ctime)?;
		table.set("mtime", self.mtime)?;
		table.set("size", self.size)?;

		table.set("content", self.content)?;

		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
