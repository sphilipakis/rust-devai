// region:    --- Modules

mod app_event_handler;
mod core;
mod event;
mod styles;
mod support;
mod term_reader;
mod views;

// -- Flatten for core::tui
use core::*;
use views::*;

// -- Only export
pub use core::start_tui;

// endregion: --- Modules
