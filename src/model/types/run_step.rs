use crate::model::ScalarEnum;
use macro_rules_attribute as mra;

// Simple wrapper for SQLite Stage
#[mra::derive(Debug, ScalarEnum!)]
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
