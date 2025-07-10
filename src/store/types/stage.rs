use crate::derive_simple_enum_type;

// Simple wrapper for SQLite Stage
derive_simple_enum_type! {
pub enum Stage {
	BeforeAll,
	Data,
	Ai,
	Output,
	AfterAll,
}
}

impl Stage {
	pub fn from_str(name: &str) -> Option<Stage> {
		match name {
			"BeforeAll" => Some(Stage::BeforeAll),
			"Data" => Some(Stage::Data),
			"Ai" => Some(Stage::Ai),
			"Output" => Some(Stage::Output),
			"AfterAll" => Some(Stage::AfterAll),
			_ => None,
		}
	}
}
