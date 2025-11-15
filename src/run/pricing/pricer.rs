use super::ModelPricing;
use super::data::PROVIDERS;
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
	// Since not api from genai yet, extract the eventual namespace of the modelname
	let model_name = normalize_model_name(model_name);

	let provider_type = normalize_provider_type(provider_type);

	// Find the provider
	// Find the model within the provider (longest start_with)
	let model = find_model_entry(provider_type, model_name)?;

	// -- Extract prompt related tokens and pricing
	let prompt_tokens = usage.prompt_tokens.unwrap_or(0) as f64;
	// Extract cached vs normal prompt tokens
	let (prompt_tokens_normal, prompt_cached_tokens, prompt_cache_creation_tokens) = match &usage.prompt_tokens_details
	{
		Some(details) => {
			let cached = details.cached_tokens.unwrap_or(0) as f64;
			let cache_creation_tokens = details.cache_creation_tokens.unwrap_or(0) as f64;
			let normal = prompt_tokens - cached - cache_creation_tokens;
			(normal, cached, cache_creation_tokens)
		}
		None => (prompt_tokens, 0.0, 0.0),
	};
	// Extract prompt prices
	let price_prompt_normal = model.input_normal;
	let price_prompt_cached = model.input_cached.unwrap_or(price_prompt_normal);
	// Note:  For now, for cache_creation_tokens assume * 1.25 for cache_creation_tokens (which is Anthropic rules, and this is only anthropic data)
	let price_prompt_cache_creation = 1.25 * price_prompt_normal;

	// -- Extract completion related tokens and pricing
	let completion_tokens = usage.completion_tokens.unwrap_or(0) as f64;
	let (completion_tokens_normal, completion_tokens_reasoning) = if let Some(reasoning_tokens) = usage
		.completion_tokens_details
		.as_ref()
		.and_then(|v| v.reasoning_tokens.map(|v| v as f64))
	{
		(completion_tokens - reasoning_tokens, reasoning_tokens)
	} else {
		(completion_tokens, 0.)
	};
	let price_completion_normal = model.output_normal;
	let price_completion_reasoning = model.output_reasoning.unwrap_or(price_completion_normal);

	// NOTE:
	let price = (prompt_tokens_normal * price_prompt_normal)
		+ (prompt_cached_tokens * price_prompt_cached)
		+ (prompt_cache_creation_tokens * price_prompt_cache_creation)
		+ (completion_tokens_normal * price_completion_normal)
		+ (completion_tokens_reasoning * price_completion_reasoning);

	// The price are per million tokens
	let price = price / 1_000_000.0;

	let price = (price * 10_000.0).round() / 10_000.0;

	Some(price)
}

pub fn model_pricing(provider_type: &str, model_name: &str) -> Option<ModelPricing> {
	let model_name = normalize_model_name(model_name);

	let provider_type = normalize_provider_type(provider_type);

	find_model_entry(provider_type, model_name).copied()
}

fn normalize_model_name(model_name: &str) -> &str {
	match model_name.split_once("::") {
		Some((_, after)) => after,
		None => model_name,
	}
}

fn normalize_provider_type(provider_type: &str) -> &str {
	if provider_type == "openai_resp" {
		"openai"
	} else {
		provider_type
	}
}

fn find_model_entry(provider_type: &str, model_name: &str) -> Option<&'static ModelPricing> {
	let provider = PROVIDERS.iter().find(|p| p.name == provider_type)?;

	let mut model: Option<&ModelPricing> = None;
	for m in provider.models.iter() {
		if model_name.starts_with(m.name) {
			let current_len = model.map(|m| m.name.len()).unwrap_or(0);
			if current_len < m.name.len() {
				model = Some(m)
			}
		}
	}

	model
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
		// ModelPricing {
		// 	name: "gpt-4o",
		// 	input_cached: Some(1.25),
		// 	input_normal: 2.5,
		// 	output_normal: 10.0,
		// 	output_reasoning: None,
		// },
		// Calculate expected: (1000 * 2.5 / 1_000_000) + (500 * 10.0 / 1_000_000)
		let expected = 0.0025 + 0.005; // 0.0075
		assert!((price - expected).abs() < f64::EPSILON);

		Ok(())
	}

	#[test]
	fn test_pricing_pricer_price_it_with_cached() -> Result<()> {
		// -- Setup & Fixtures
		let fx_prompt_normal_tokens = 1000;
		let fx_completion_tokens = 500;
		let fx_cached_tokens = 400;
		let usage = Usage {
			prompt_tokens: Some(fx_prompt_normal_tokens + fx_cached_tokens),
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

		// Calculate expected:
		let cached = fx_cached_tokens as f64 * 0.075 / 1_000_000.0;
		let prompt = fx_prompt_normal_tokens as f64 * 0.150 / 1_000_000.0;
		let completion = fx_completion_tokens as f64 * 0.6 / 1_000_000.0;
		let expected = cached + prompt + completion;
		let expected = (expected * 10_000.0).round() / 10_000.0;
		assert!((price - expected).abs() < f64::EPSILON);

		Ok(())
	}

	#[test]
	fn test_pricing_pricer_price_it_with_cached_no_cached_price() -> Result<()> {
		// -- Setup & Fixtures
		let fx_prompt_normal_tokens = 1000;
		let fx_cached_tokens = 400;
		let fx_completion_tokens = 500;
		let usage = Usage {
			prompt_tokens: Some(fx_prompt_normal_tokens + fx_cached_tokens),
			completion_tokens: Some(fx_completion_tokens),
			prompt_tokens_details: Some(PromptTokensDetails {
				cached_tokens: Some(fx_cached_tokens),
				audio_tokens: None,
				cache_creation_tokens: None,
			}),
			..Default::default()
		};

		// -- Exec
		// Test with a model that has input_cached: None (e.g., groq model)
		let price = price_it("gemini", "gemini-2.5-pro", &usage);

		// -- Check
		let price = price.ok_or("Should have price")?;

		// Calculate expected: cached tokens should use input_normal price
		let cached = fx_cached_tokens as f64 * 0.31 / 1_000_000.0;
		let prompt = fx_prompt_normal_tokens as f64 * 1.25 / 1_000_000.0;
		let completion = fx_completion_tokens as f64 * 10. / 1_000_000.0;
		let expected = cached + prompt + completion;
		let expected = (expected * 10_000.0).round() / 10_000.0;
		assert!((price - expected).abs() < f64::EPSILON);

		Ok(())
	}

	#[test]
	fn test_pricing_pricer_price_it_with_cache_creation() -> Result<()> {
		// -- Setup & Fixtures
		let fx_prompt_normal_tokens = 1000;
		let fx_cache_creation_tokens = 200;
		let fx_cached_tokens = 400;
		let fx_completion_tokens = 500;
		let usage = Usage {
			prompt_tokens: Some(fx_prompt_normal_tokens + fx_cache_creation_tokens + fx_cached_tokens),
			completion_tokens: Some(fx_completion_tokens),
			prompt_tokens_details: Some(PromptTokensDetails {
				cached_tokens: Some(fx_cached_tokens),
				cache_creation_tokens: Some(fx_cache_creation_tokens),
				audio_tokens: None,
			}),
			..Default::default()
		};

		// -- Exec
		// Test with an Anthropic model which uses cache_creation_tokens
		let price = price_it("anthropic", "claude-3-5-sonnet", &usage);

		// -- Check
		assert!(price.is_some());
		let price = price.unwrap();

		// Calculate expected:
		let cached = fx_cached_tokens as f64 * 0.3 / 1_000_000.0;
		// NOTE: cache_creation uses input_normal * 1.25
		let cache_creation = fx_cache_creation_tokens as f64 * 1.25 * 3.0 / 1_000_000.0;
		let prompt = fx_prompt_normal_tokens as f64 * 3.0 / 1_000_000.0;
		let completion = fx_completion_tokens as f64 * 15.0 / 1_000_000.0;
		let expected = cached + cache_creation + prompt + completion;
		let expected = (expected * 10_000.0).round() / 10_000.0; // 0.00012 + 0.00075 + 0.003 + 0.0075 = 0.01137 -> 0.0114
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
