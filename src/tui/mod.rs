// region:    --- Modules

mod core;
mod event;
mod styles;
mod support;
mod views;

// -- Flatten for core::tui
use core::*;
use views::*;

// -- Only export
pub use core::start_tui;

// endregion: --- Modules
