// region:    --- Modules

mod core;
mod support;
mod view;

// -- Flatten for core::tui
use core::*;
use view::*;

// -- Only export
pub use core::start_tui;

// endregion: --- Modules
