use crate::derive_simple_enum_type;

// Simple wrapper for SQLite Stage
derive_simple_enum_type! {
pub enum RunStep {
	Start,
	End,
	BaStart,
	BaEnd,
	TasksStart, // First tasks start
	TasksEnd, // All tasks ended
	TaskStart,
	TaskEnd,
	DtStart,
	DtEnd,
	OtStart,
	OtEnd,
	AaStart,
	AaEnd,
}
}
