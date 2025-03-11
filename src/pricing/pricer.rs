use crate::pricing::data::PROVIDERS;
use genai::chat::Usage;

/// Calculates the price for a given provider type, model name, and usage.
///
/// # Arguments
/// * `provider_type` - The type of provider (e.g., "openai", "groq", "gemini", "deepseek", "anthropic")
/// * `model_name` - The name of the model to price
/// * `usage` - The token usage information
///
/// # Returns
/// * `Option<f64>` - The calculated price in USD, or None if the provider or model was not found
pub fn price_it(provider_type: &str, model_name: &str, usage: &Usage) -> Option<f64> {
	// Find the provider
	let provider = PROVIDERS.iter().find(|p| p.name == provider_type)?;

	// Find the model within the provider (longest start_with)
	let model = provider.models.iter().find(|m| m.name.starts_with(model_name))?;

	// Extract token counts from usage
	let prompt_tokens = usage.prompt_tokens.unwrap_or(0) as f64;
	let completion_tokens = usage.completion_tokens.unwrap_or(0) as f64;

	// Calculate cached vs normal prompt tokens
	let (cached_tokens, cache_creation_tokens) = match &usage.prompt_tokens_details {
		Some(details) => {
			let cached = details.cached_tokens.unwrap_or(0) as f64;
			let cache_creation_tokens = details.cache_creation_tokens.unwrap_or(0) as f64;
			(cached, cache_creation_tokens)
		}
		None => (0.0, 0.0),
	};

	// Calculate price (convert from per million tokens to actual price)
	// NOTE: For now, hack the * 1.25 for cache_creation_tokens (which is Anthropic rules, and this is only anthropic data)
	let price = (cached_tokens * model.input_cached / 1_000_000.0)
		+ (cache_creation_tokens * 1.25 * model.input_normal / 1_000_000.0)
		+ (prompt_tokens * model.input_normal / 1_000_000.0)
		+ (completion_tokens * model.output / 1_000_000.0);

	let price = (price * 10_000.0).round() / 10_000.0;

	Some(price)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use genai::chat::{PromptTokensDetails, Usage};

	#[test]
	fn test_pricing_pricer_price_it_simple() -> Result<()> {
		// -- Setup & Fixtures
		let usage = Usage {
			prompt_tokens: Some(1000),
			completion_tokens: Some(500),
			prompt_tokens_details: None,
			..Default::default()
		};

		// -- Exec
		let price = price_it("openai", "gpt-4o", &usage);

		// -- Check
		assert!(price.is_some());
		let price = price.unwrap();
		// Calculate expected: (1000 * 2.5 / 1_000_000) + (500 * 10.0 / 1_000_000)
		let expected = 0.0025 + 0.005; // 0.0075
		assert!((price - expected).abs() < f64::EPSILON);

		Ok(())
	}

	#[test]
	fn test_pricing_pricer_price_it_with_cached() -> Result<()> {
		// -- Setup & Fixtures
		let fx_prompt_tokens = 1000;
		let fx_completion_tokens = 500;
		let fx_cached_tokens = 400;
		let usage = Usage {
			prompt_tokens: Some(fx_prompt_tokens),
			completion_tokens: Some(fx_completion_tokens),
			prompt_tokens_details: Some(PromptTokensDetails {
				cached_tokens: Some(fx_cached_tokens),
				audio_tokens: None,
				cache_creation_tokens: None,
			}),
			..Default::default()
		};

		// -- Exec
		let price = price_it("openai", "gpt-4o-mini", &usage);

		// -- Check
		assert!(price.is_some());
		let price = price.unwrap();
		// ModelPricing {
		// 	name: "gpt-4o-mini",
		// 	input_cached: 0.075,
		// 	input_normal: 0.150,
		// 	output: 0.600,
		// },

		// Calculate expected:
		let cached = fx_cached_tokens as f64 * 0.075 / 1_000_000.0;
		let prompt = fx_prompt_tokens as f64 * 0.150 / 1_000_000.0;
		let completion = fx_completion_tokens as f64 * 0.6 / 1_000_000.0;
		let expected = cached + prompt + completion;
		let expected = (expected * 10_000.0).round() / 10_000.0;
		assert!((price - expected).abs() < f64::EPSILON);

		Ok(())
	}

	#[test]
	fn test_pricing_pricer_price_it_unknown_provider() -> Result<()> {
		// -- Setup & Fixtures
		let usage = Usage {
			prompt_tokens: Some(1000),
			completion_tokens: Some(500),
			..Default::default()
		};

		// -- Exec
		let price = price_it("unknown_provider", "gpt-4o", &usage);

		// -- Check
		assert!(price.is_none());

		Ok(())
	}

	#[test]
	fn test_pricing_pricer_price_it_unknown_model() -> Result<()> {
		// -- Setup & Fixtures
		let usage = Usage {
			prompt_tokens: Some(1000),
			completion_tokens: Some(500),
			..Default::default()
		};

		// -- Exec
		let price = price_it("openai", "unknown_model", &usage);

		// -- Check
		assert!(price.is_none());

		Ok(())
	}
}

// endregion: --- Tests
