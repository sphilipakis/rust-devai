#[derive(Debug)]
pub struct ModelPricing {
	pub name: &'static str,
	pub input_cached: f64,
	pub input_normal: f64,
	pub output: f64,
}

#[derive(Debug)]
pub struct Provider {
	pub name: &'static str,
	pub models: &'static [ModelPricing],
}

// Define OpenAI pricing
const OPENAI_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "gpt-4.1",
		input_cached: 0.5,
		input_normal: 2.0,
		output: 8.0,
	},
	ModelPricing {
		name: "gpt-4.1-mini",
		input_cached: 0.1,
		input_normal: 0.4,
		output: 1.6,
	},
	ModelPricing {
		name: "gpt-4.1-nano",
		input_cached: 0.025,
		input_normal: 0.1,
		output: 0.4,
	},
	ModelPricing {
		name: "gpt-4o",
		input_cached: 1.25,
		input_normal: 2.5,
		output: 10.0,
	},
	ModelPricing {
		name: "gpt-4o-mini",
		input_cached: 0.075,
		input_normal: 0.15,
		output: 0.6,
	},
	ModelPricing {
		name: "o1",
		input_cached: 7.5,
		input_normal: 15.0,
		output: 60.0,
	},
	ModelPricing {
		name: "o1-pro",
		input_cached: 0.0, // Mapped from null
		input_normal: 150.0,
		output: 600.0,
	},
	ModelPricing {
		name: "o3-mini",
		input_cached: 0.55,
		input_normal: 1.1,
		output: 4.4,
	},
	ModelPricing {
		name: "o1-mini",
		input_cached: 0.55,
		input_normal: 1.1,
		output: 4.4,
	},
];

const OPENAI: Provider = Provider {
	name: "openai",
	models: OPENAI_MODELS,
};

// Define Groq pricing
const GROQ_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "deepseek-r1-distill-llama-70b",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.75,
		output: 0.99,
	},
	ModelPricing {
		name: "deepseek-r1-distill-qwen-32b-128k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.69,
		output: 0.69,
	},
	ModelPricing {
		name: "qwen-2.5-32b-instruct-128k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.79,
		output: 0.79,
	},
	ModelPricing {
		name: "qwen-2.5-coder-32b-instruct-128k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.79,
		output: 0.79,
	},
	ModelPricing {
		name: "mistral-saba-24b",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.79,
		output: 0.79,
	},
	ModelPricing {
		name: "llama-3.3-70b-versatile-128k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.59,
		output: 0.79,
	},
	ModelPricing {
		name: "llama-3.1-8b-instant-128k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.05,
		output: 0.08,
	},
	ModelPricing {
		name: "llama-3-70b-8k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.59,
		output: 0.79,
	},
	ModelPricing {
		name: "llama-3-8b-8k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.05,
		output: 0.08,
	},
	ModelPricing {
		name: "mixtral-8x7b-instruct-32k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.24,
		output: 0.24,
	},
	ModelPricing {
		name: "gemma-2-9b-8k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.2,
		output: 0.2,
	},
	ModelPricing {
		name: "llama-guard-3-8b-8k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.2,
		output: 0.2,
	},
	ModelPricing {
		name: "llama-3.3-70b-specdec-8k",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.59,
		output: 0.99,
	},
];

const GROQ: Provider = Provider {
	name: "groq",
	models: GROQ_MODELS,
};

// Define Gemini pricing
const GEMINI_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "gemini-2.5-pro", // Renamed from gemini-2.5-pro-preview
		input_cached: 0.0,      // Mapped from null
		input_normal: 1.25,
		output: 10.0,
	},
	ModelPricing {
		name: "gemini-2.0-flash",
		input_cached: 0.025,
		input_normal: 0.1,
		output: 0.4,
	},
	ModelPricing {
		name: "gemini-2.0-flash-lite",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.075,
		output: 0.3,
	},
	ModelPricing {
		name: "imagen-3",
		input_cached: 0.0,  // Mapped from null
		input_normal: 0.03, // Updated from 0.0
		output: 0.03,
	},
	ModelPricing {
		name: "gemma-3",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.0,
		output: 0.0,
	},
	ModelPricing {
		name: "gemini-1.5-flash",
		input_cached: 0.01875,
		input_normal: 0.075,
		output: 0.3,
	},
	ModelPricing {
		name: "gemini-1.5-flash-8b",
		input_cached: 0.01,
		input_normal: 0.0375,
		output: 0.15,
	},
	ModelPricing {
		name: "gemini-1.5-pro",
		input_cached: 0.0, // Mapped from null
		input_normal: 1.25,
		output: 5.0,
	},
	ModelPricing {
		name: "text-embedding-004",
		input_cached: 0.0, // Mapped from null
		input_normal: 0.0,
		output: 0.0,
	},
];

const GEMINI: Provider = Provider {
	name: "gemini",
	models: GEMINI_MODELS,
};

// Define Deepseek pricing
const DEEPSEEK_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "deepseek-chat",
		input_cached: 0.07,
		input_normal: 0.27,
		output: 1.1,
	},
	ModelPricing {
		name: "deepseek-reasoner",
		input_cached: 0.14,
		input_normal: 0.55,
		output: 2.19,
	},
];

const DEEPSEEK: Provider = Provider {
	name: "deepseek",
	models: DEEPSEEK_MODELS,
};

// Define Anthropic pricing
const ANTHROPIC_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "claude-3-7-sonnet",
		input_cached: 0.3,
		input_normal: 3.0,
		output: 15.0,
	},
	ModelPricing {
		name: "claude-3-5-haiku",
		input_cached: 0.08,
		input_normal: 0.8,
		output: 4.0,
	},
	ModelPricing {
		name: "claude-3-opus",
		input_cached: 1.5,
		input_normal: 15.0,
		output: 75.0,
	},
	ModelPricing {
		name: "claude-3-5-sonnet",
		input_cached: 0.3,
		input_normal: 3.0,
		output: 15.0,
	},
	ModelPricing {
		name: "claude-3-haiku",
		input_cached: 0.03,
		input_normal: 0.25,
		output: 1.25,
	},
];

const ANTHROPIC: Provider = Provider {
	name: "anthropic",
	models: ANTHROPIC_MODELS,
};

pub const PROVIDERS: &[Provider] = &[OPENAI, GROQ, GEMINI, DEEPSEEK, ANTHROPIC];
