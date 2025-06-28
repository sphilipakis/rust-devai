use crate::dir_context::path_consts::SUPPORT_PACK;
use crate::dir_context::{DirContext, find_pack_dirs};
use crate::types::{PackRef, PackRefSubPathScope};
use crate::{Error, Result};
use simple_fs::SPath;

/// This resolve the base path (might not exists) of a PackRef
/// It also support the `support` scheme with `$base` and `$workspace`
///
/// The base_pat is the bpas before the `sub_path`
///
/// Note: This path might not exists
///
/// ## Examples
///
/// - `pro@rust10x/guide/base/some.md` `~/.aipack-base/pack/installed/pro/rust10x` (so the dir)
/// - `pro@rust10x/guide/base/**/*.md` `~/.aipack-base/pack/installed/pro/rust10x` (so the dir)
/// - `pro@coder$workspace/so/data.md` `.aipack/support/pack/pro/coder`
pub fn resolve_pack_ref_base_path(dir_context: &DirContext, pack_ref: &PackRef) -> Result<SPath> {
	match pack_ref.sub_path_scope {
		// -- If we have the pack dir,
		PackRefSubPathScope::PackDir => {
			// -- Get the base path
			let pack_dirs = find_pack_dirs(dir_context, pack_ref)?;
			let Some(pack_dir) = pack_dirs.into_iter().next() else {
				return Err(Error::custom(format!("Cannot find the base path for {pack_ref}")));
			};

			Ok(pack_dir.path)
		}
		// -- Support $base
		PackRefSubPathScope::BaseSupport => {
			let aipack_base_dir = dir_context.aipack_paths().aipack_base_dir();
			let path = join_support_pack_ref(aipack_base_dir.path(), &pack_ref.identity_as_path());
			Ok(path)
		}
		// -- Support $workspace
		PackRefSubPathScope::WorkspaceSupport => {
			let wks_dir = dir_context.aipack_paths().aipack_wks_dir().ok_or_else(|| {
				format!("Cannot load reference support file in workspace for '{pack_ref}' because no workspace")
			})?;
			let path = join_support_pack_ref(wks_dir.path(), &pack_ref.identity_as_path());
			Ok(path)
		}
	}
}

/// aipack_path is the .aipack/ or ~/.aipack-base/ path.
pub fn join_support_pack_ref(aipack_path: &SPath, identity_path: &SPath) -> SPath {
	aipack_path.join(SUPPORT_PACK).join(identity_path)
}
