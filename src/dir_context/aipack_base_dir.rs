use crate::Result;
use crate::dir_context::path_consts::AIPACK_BASE;
use camino::Utf8PathBuf;
use home::home_dir;
use simple_fs::SPath;
use std::ops::Deref;

/// BaseAipackPath is the typed wrapper of the `~/.aipack-base` absolute path
#[derive(Debug, Clone)]
pub struct AipackBaseDir {
	path: SPath,
}

impl AipackBaseDir {
	/// Build the absolute path for `~/.aipack-base/`
	/// NOTE: This does not test if it exists
	/// Should be use at the only way to get the aipack base dir
	pub fn new() -> Result<Self> {
		Ok(Self {
			path: aipack_base_dir()?,
		})
	}

	pub fn path(&self) -> &SPath {
		&self.path
	}
}

/// Pathroughts to SPath
impl AipackBaseDir {
	pub fn exists(&self) -> bool {
		self.path.exists()
	}
	pub fn join(&self, leaf_path: impl Into<Utf8PathBuf>) -> SPath {
		self.path.join(leaf_path)
	}
}

impl AsRef<SPath> for AipackBaseDir {
	fn as_ref(&self) -> &SPath {
		&self.path
	}
}

impl Deref for AipackBaseDir {
	type Target = SPath;

	fn deref(&self) -> &Self::Target {
		&self.path
	}
}

#[cfg(test)]
impl AipackBaseDir {
	pub fn new_for_test(path: impl Into<SPath>) -> Result<Self> {
		let path = path.into();
		Ok(Self { path })
	}
}

/// This returns the `~/.aipack-base` full path
///
/// NOTE: This does NOT create or test if the path exists
///
fn aipack_base_dir() -> Result<SPath> {
	let home_dir = home_dir().ok_or("No Home Dir Found, cannot init ./aipack-base")?;
	if !home_dir.exists() {
		Err(format!("Home dir '{}' does not exist", home_dir.to_string_lossy()))?;
	}
	let home_dir = SPath::from_std_path_buf(home_dir)?;

	let base_dir = home_dir.join(AIPACK_BASE);

	Ok(base_dir)
}
