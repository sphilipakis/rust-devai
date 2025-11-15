// region:    --- Modules

mod data;
mod pricer;
mod pricing_types;

use pricing_types::*;

// endregion: --- Modules

// region:    --- Public API

pub use pricer::price_it;

// endregion: --- Public API
