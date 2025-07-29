// region:    --- Modules

mod asserts;
mod base;
mod loaders;
mod lua_test_support;
mod runners;
mod seeders;
mod test_files;

pub use asserts::*;
pub use base::*;
pub use loaders::*;
pub use lua_test_support::*;
pub use runners::*;
pub use seeders::*;
pub use test_files::*;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

// endregion: --- Modules
