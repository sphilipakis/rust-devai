use crate::Result;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use simple_fs::SPath;

/// The path resolver function to be used
impl Runtime {
	/// The default path resolver using WksDir when relative path
	pub fn resolve_path_default(&self, path: SPath, base_dir: Option<&SPath>) -> Result<SPath> {
		self.dir_context()
			.resolve_path(self.session(), path, PathResolver::WksDir, base_dir)
	}
}
