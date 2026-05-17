// region:    --- Modules

mod pricer;
mod pricing_types;

// endregion: --- Modules

// region:    --- Public API
pub use pricer::{model_pricing, price_it};
pub use pricing_types::ModelPricing;

// endregion: --- Public API
