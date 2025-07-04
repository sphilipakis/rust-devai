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
