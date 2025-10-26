use crate::derive_simple_enum_type;
use macro_rules_attribute::apply;

// Simple wrapper for SQLite Stage
#[apply(derive_simple_enum_type)]
pub enum RunStep {
	Start,

	// -- Before All
	BaStart,
	BaEnd,
	TasksStart, // First tasks start
	TasksEnd,   // All tasks ended

	// -- Task
	TaskStart,
	TaskEnd,
	TaskDataStart,
	TaskDataEnd,
	TaskAiStart,
	TaskAiGenStart,
	TaskAiGenEnd,
	TaskAiEnd,
	TaskOutputStart,
	TaskOutputEnd,

	// -- After All
	AaStart,
	AaEnd,

	End,
}
