// region:    --- Modules

mod support;

mod xelf_setup;
mod xelf_update;
mod xelf_update_nix; // Added new module for Nix-like OS updates

pub use xelf_setup::exec_xelf_setup;
pub use xelf_update::exec_xelf_update;

// endregion: --- Modules
