// region:    --- Modules

mod file_change;
mod file_common;
mod file_hash;
mod file_html;
mod file_json;
mod file_md;
mod support;

mod init;

pub use init::*;
// Note: The individual functions from file_hash.rs (like file_hash_sha256)
// are pub(super) and will be explicitly registered in init.rs,
// so no `pub use file_hash::*;` is needed here.
