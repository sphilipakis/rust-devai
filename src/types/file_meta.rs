use mlua::{IntoLua, Lua};
use serde::Serialize;
use simple_fs::{SFile, SPath};

/// The FileMeta object contains the metadata of a file but not its content.
/// The created_epoch_us, modified_epoch_us, and size metadata are generally loaded,
/// but this can be turned off when listing files using the `with_meta = false` option.
#[derive(Debug, Serialize)]
pub struct FileMeta {
	path: String,
	/// The dir/parent path of this file from path (will be empty if no parent of the rel path)
	dir: String,
	name: String,
	stem: String,
	ext: String,

	created_epoch_us: Option<i64>, // seconds since epoch, or nil in Lua
	modified_epoch_us: Option<i64>,
	size: Option<i64>, // size in bytes
}

impl FileMeta {
	/// - with_meta: when true, will attempt to get the file meta. Will ignore if error
	pub fn new(path: impl Into<SPath>, with_meta: bool) -> Self {
		let path: SPath = path.into();
		let meta = path.meta().ok();
		let mut res = FileMeta::from(path);
		if with_meta {
			if let Some(meta) = meta {
				res.created_epoch_us = Some(meta.created_epoch_us);
				res.modified_epoch_us = Some(meta.modified_epoch_us);
				res.size = Some(meta.size);
			}
		}
		res
	}
}

impl From<&SPath> for FileMeta {
	fn from(file: &SPath) -> Self {
		let dir = file.parent().map(|p| p.to_string()).unwrap_or_default();
		FileMeta {
			path: file.to_string(),
			name: file.name().to_string(),
			dir,
			stem: file.stem().to_string(),
			ext: file.ext().to_string(),
			// -- when created _with_meta
			created_epoch_us: None,
			modified_epoch_us: None,
			size: None,
		}
	}
}

impl From<SPath> for FileMeta {
	fn from(spath: SPath) -> Self {
		From::<&SPath>::from(&spath)
	}
}

impl From<SFile> for FileMeta {
	fn from(file: SFile) -> Self {
		let spath: SPath = file.into();
		From::<SPath>::from(spath)
	}
}

// region:    --- Lua

impl IntoLua for FileMeta {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("path", self.path)?;
		table.set("dir", self.dir)?;
		table.set("name", self.name)?;
		table.set("stem", self.stem)?;
		table.set("ext", self.ext)?;
		if let Some(created_epoch_us) = self.created_epoch_us {
			table.set("created_epoch_us", created_epoch_us)?;
		}
		if let Some(modified_epoch_us) = self.modified_epoch_us {
			table.set("modified_epoch_us", modified_epoch_us)?;
		}
		if let Some(size) = self.size {
			table.set("size", size)?;
		}
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
