#![allow(unused)] // For early development.

#[derive(Debug)]
pub struct ModelPricing {
	pub name: &'static str,
	pub input_cached: Option<f64>,
	pub input_normal: f64,
	pub output_normal: f64,
	pub output_reasoning: Option<f64>,
}

#[derive(Debug)]
pub struct Provider {
	pub name: &'static str,
	pub models: &'static [ModelPricing],
}

// Define Anthropic pricing
const ANTHROPIC_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "claude-opus-4",
		input_cached: Some(1.5),
		input_normal: 15.0,
		output_normal: 75.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-sonnet-4",
		input_cached: Some(0.3),
		input_normal: 3.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-3-7-sonnet",
		input_cached: Some(0.3),
		input_normal: 3.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-3-5-sonnet",
		input_cached: Some(0.3),
		input_normal: 3.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-3-5-haiku",
		input_cached: Some(0.08),
		input_normal: 0.8,
		output_normal: 4.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-3-opus",
		input_cached: Some(1.5),
		input_normal: 15.0,
		output_normal: 75.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-3-haiku",
		input_cached: Some(0.03),
		input_normal: 0.25,
		output_normal: 1.25,
		output_reasoning: None,
	},
];

const ANTHROPIC: Provider = Provider {
	name: "anthropic",
	models: ANTHROPIC_MODELS,
};

// Define Deepseek pricing
const DEEPSEEK_MODELS: &[ModelPricing] = &[
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

const DEEPSEEK: Provider = Provider {
	name: "deepseek",
	models: DEEPSEEK_MODELS,
};

// Define Gemini pricing
const GEMINI_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "gemini-2.5-pro",
		input_cached: Some(0.31),
		input_normal: 1.25,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.5-flash",
		input_cached: Some(0.075),
		input_normal: 0.3,
		output_normal: 2.5,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.5-flash-lite",
		input_cached: Some(0.025),
		input_normal: 0.1,
		output_normal: 0.4,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.0-flash",
		input_cached: Some(0.025),
		input_normal: 0.1,
		output_normal: 0.4,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.0-flash-lite",
		input_cached: None,
		input_normal: 0.075,
		output_normal: 0.3,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-1.5-flash",
		input_cached: Some(0.01875),
		input_normal: 0.075,
		output_normal: 0.3,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-1.5-flash-8b",
		input_cached: Some(0.01),
		input_normal: 0.0375,
		output_normal: 0.15,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-1.5-pro",
		input_cached: Some(0.3125),
		input_normal: 1.25,
		output_normal: 5.0,
		output_reasoning: None,
	},
];

const GEMINI: Provider = Provider {
	name: "gemini",
	models: GEMINI_MODELS,
};

// Define Groq pricing
const GROQ_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "deepseek-r1-distill-llama-70b",
		input_cached: None,
		input_normal: 0.75,
		output_normal: 0.99,
		output_reasoning: None,
	},
	ModelPricing {
		name: "deepseek-r1-distill-qwen-32b-128k",
		input_cached: None,
		input_normal: 0.69,
		output_normal: 0.69,
		output_reasoning: None,
	},
	ModelPricing {
		name: "qwen-2.5-32b-instruct-128k",
		input_cached: None,
		input_normal: 0.79,
		output_normal: 0.79,
		output_reasoning: None,
	},
	ModelPricing {
		name: "qwen-2.5-coder-32b-instruct-128k",
		input_cached: None,
		input_normal: 0.79,
		output_normal: 0.79,
		output_reasoning: None,
	},
	ModelPricing {
		name: "qwen-qwq-32b-(preview)-128k",
		input_cached: None,
		input_normal: 0.29,
		output_normal: 0.39,
		output_reasoning: None,
	},
	ModelPricing {
		name: "mistral-saba-24b",
		input_cached: None,
		input_normal: 0.79,
		output_normal: 0.79,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-3.2-1b-(preview)-8k",
		input_cached: None,
		input_normal: 0.04,
		output_normal: 0.04,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-3.2-3b-(preview)-8k",
		input_cached: None,
		input_normal: 0.06,
		output_normal: 0.06,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-3.3-70b-versatile-128k",
		input_cached: None,
		input_normal: 0.59,
		output_normal: 0.79,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-3.1-8b-instant-128k",
		input_cached: None,
		input_normal: 0.05,
		output_normal: 0.08,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-3-70b-8k",
		input_cached: None,
		input_normal: 0.59,
		output_normal: 0.79,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-3-8b-8k",
		input_cached: None,
		input_normal: 0.05,
		output_normal: 0.08,
		output_reasoning: None,
	},
	ModelPricing {
		name: "mixtral-8x7b-instruct-32k",
		input_cached: None,
		input_normal: 0.24,
		output_normal: 0.24,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemma-2-9b-8k",
		input_cached: None,
		input_normal: 0.2,
		output_normal: 0.2,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-guard-3-8b-8k",
		input_cached: None,
		input_normal: 0.2,
		output_normal: 0.2,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-3.3-70b-specdec-8k",
		input_cached: None,
		input_normal: 0.59,
		output_normal: 0.99,
		output_reasoning: None,
	},
];

const GROQ: Provider = Provider {
	name: "groq",
	models: GROQ_MODELS,
};

// Define OpenAI pricing
const OPENAI_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "gpt-4.1",
		input_cached: Some(0.5),
		input_normal: 2.0,
		output_normal: 8.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-4.1-mini",
		input_cached: Some(0.1),
		input_normal: 0.4,
		output_normal: 1.6,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-4.1-nano",
		input_cached: Some(0.025),
		input_normal: 0.1,
		output_normal: 0.4,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-4.5-preview",
		input_cached: Some(37.5),
		input_normal: 75.0,
		output_normal: 150.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-4o",
		input_cached: Some(1.25),
		input_normal: 2.5,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-4o-realtime-preview",
		input_cached: Some(2.5),
		input_normal: 5.0,
		output_normal: 20.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-4o-mini",
		input_cached: Some(0.075),
		input_normal: 0.15,
		output_normal: 0.6,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-4o-mini-realtime-preview",
		input_cached: Some(0.3),
		input_normal: 0.6,
		output_normal: 2.4,
		output_reasoning: None,
	},
	ModelPricing {
		name: "o1",
		input_cached: Some(7.5),
		input_normal: 15.0,
		output_normal: 60.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "o1-pro",
		input_cached: None,
		input_normal: 150.0,
		output_normal: 600.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "o3-pro",
		input_cached: None,
		input_normal: 20.0,
		output_normal: 80.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "o3",
		input_cached: Some(0.5),
		input_normal: 2.0,
		output_normal: 8.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "o4-mini",
		input_cached: Some(0.275),
		input_normal: 1.1,
		output_normal: 4.4,
		output_reasoning: None,
	},
	ModelPricing {
		name: "o3-mini",
		input_cached: Some(0.55),
		input_normal: 1.1,
		output_normal: 4.4,
		output_reasoning: None,
	},
	ModelPricing {
		name: "o1-mini",
		input_cached: Some(0.55),
		input_normal: 1.1,
		output_normal: 4.4,
		output_reasoning: None,
	},
	ModelPricing {
		name: "codex-mini-latest",
		input_cached: Some(0.375),
		input_normal: 1.5,
		output_normal: 6.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-4o-mini-search-preview",
		input_cached: None,
		input_normal: 0.15,
		output_normal: 0.6,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-4o-search-preview",
		input_cached: None,
		input_normal: 2.5,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "computer-use-preview",
		input_cached: None,
		input_normal: 3.0,
		output_normal: 12.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gpt-image-1",
		input_cached: Some(1.25),
		input_normal: 5.0,
		output_normal: 0.0,
		output_reasoning: None,
	},
];

const OPENAI: Provider = Provider {
	name: "openai",
	models: OPENAI_MODELS,
};

// Define XAI pricing
const XAI_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "grok-4",
		input_cached: Some(0.75),
		input_normal: 3.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-3",
		input_cached: Some(0.75),
		input_normal: 3.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-3-mini",
		input_cached: Some(0.075),
		input_normal: 0.3,
		output_normal: 0.5,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-3-fast",
		input_cached: Some(1.25),
		input_normal: 5.0,
		output_normal: 25.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-3-mini-fast",
		input_cached: Some(0.15),
		input_normal: 0.6,
		output_normal: 4.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-beta",
		input_cached: None,
		input_normal: 5.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-2-image-gen",
		input_cached: None,
		input_normal: 0.0,
		output_normal: 0.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-2-vision-1212",
		input_cached: None,
		input_normal: 2.0,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-2-1212",
		input_cached: None,
		input_normal: 2.0,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-vision-beta",
		input_cached: None,
		input_normal: 5.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-2-image-1212",
		input_cached: None,
		input_normal: 0.0,
		output_normal: 0.07,
		output_reasoning: None,
	},
];

const XAI: Provider = Provider {
	name: "xai",
	models: XAI_MODELS,
};

pub const PROVIDERS: &[Provider] = &[OPENAI, GROQ, GEMINI, DEEPSEEK, ANTHROPIC, XAI];
