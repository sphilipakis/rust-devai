// region:    --- Modules

mod core;
mod support;
mod view;

// -- Flatten for core::tui
// -- Only export
pub use core::start_tui;
use core::*;
use view::*;

// endregion: --- Modules
