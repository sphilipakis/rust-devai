// region:    --- Modules

mod db;
mod derive_aliases;
mod error;
mod model_manager;
mod types;

use derive_aliases::*;

pub use error::{Error, Result};
pub use model_manager::*;
pub use types::*;

pub mod base;

pub mod rt_model;

// endregion: --- Modules
