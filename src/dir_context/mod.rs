// region:    --- Modules

mod aipack_base_dir;
mod aipack_paths;
mod aipack_wks_dir; // Added new module
mod base;
mod pack_dir;
mod path_consts;
mod path_resolvers;

pub use aipack_base_dir::*;
pub use aipack_paths::*;
pub use aipack_wks_dir::*; // Export new type
pub use base::*;
pub use pack_dir::*;
pub use path_resolvers::*;

// endregion: --- Modules
