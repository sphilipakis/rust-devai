// region:    --- Modules

mod app_event_handlers;
mod event;
mod term_reader;
mod tui_loop;

// -- For cherry
mod app_state;
mod tui_impl;
mod ping_timer;

// -- Public
mod types;

// -- Cherry Flatten
pub use app_state::AppState;
pub use tui_impl::{AppTx, ExitTx, start_tui};
pub use ping_timer::{PingTimerTx, start_ping_timer};

// -- Public flatten
pub use types::*;

// endregion: --- Modules

