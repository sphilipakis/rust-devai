use crate::run::pricing::{ModelPricing, ProviderPricing};

pub const DEEPSEEK: ProviderPricing = ProviderPricing {
	name: "deepseek",
	models: DEEPSEEK_MODELS,
};

const DEEPSEEK_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "deepseek-v4-flash",
		input_cached: Some(0.0028),
		input_normal: 0.14,
		output_normal: 0.28,
		output_reasoning: None,
	},
	ModelPricing {
		name: "deepseek-v4-pro",
		input_cached: Some(0.0145),
		input_normal: 1.74,
		output_normal: 3.48,
		output_reasoning: None,
	},
	ModelPricing {
		name: "deepseek-chat",
		input_cached: Some(0.07),
		input_normal: 0.27,
		output_normal: 1.1,
		output_reasoning: None,
	},
	ModelPricing {
		name: "deepseek-reasoner",
		input_cached: Some(0.14),
		input_normal: 0.55,
		output_normal: 2.19,
		output_reasoning: None,
	},
];
