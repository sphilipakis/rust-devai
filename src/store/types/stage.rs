use crate::derive_simple_enum_type;
use macro_rules_attribute::apply;

// Simple wrapper for SQLite Stage
#[apply(derive_simple_enum_type)]
pub enum Stage {
	BeforeAll,
	Data,
	Ai,
	AiGen,
	Output,
	AfterAll,
}

impl Stage {
	pub fn from_str(name: &str) -> Option<Stage> {
		match name {
			"BeforeAll" => Some(Stage::BeforeAll),
			"Data" => Some(Stage::Data),
			"Ai" => Some(Stage::Ai),
			"AiGen" => Some(Stage::AiGen),
			"Output" => Some(Stage::Output),
			"AfterAll" => Some(Stage::AfterAll),
			_ => None,
		}
	}
}
