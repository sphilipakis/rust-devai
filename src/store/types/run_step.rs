use crate::derive_simple_enum_type;

// Simple wrapper for SQLite Stage
derive_simple_enum_type! {
pub enum RunStep {
	Start,

	// -- Before All
	BaStart,
	BaEnd,
	TasksStart, // First tasks start
	TasksEnd, // All tasks ended

	// -- Task
	TaskStart,
	TaskEnd,
	TaskDtStart,
	TaskDtEnd,
	TaskOtStart,
	TaskOtEnd,

	// -- After All
	AaStart,
	AaEnd,


	End,
}
}
