#[derive(Debug, Clone)]
pub enum RunEvent {
	// -- Global Run
	Start(RunTs),
	End(RunTs),
	BaStart(RunTs),

	// -- Before All
	BaPrint(RunPrint),
	BaEnd(RunTs),

	// -- Data
	DtStart { task_id: i64 },
}

impl RunEvent {
	// -- Global Run
	pub fn start(run_id: i64, time_us: i64) -> Self {
		Self::Start(RunTs { run_id, time_us })
	}
	pub fn end(run_id: i64, time_us: i64) -> Self {
		Self::End(RunTs { run_id, time_us })
	}

	// -- Before All
	pub fn ba_start(run_id: i64, time_us: i64) -> Self {
		Self::BaStart(RunTs { run_id, time_us })
	}
	pub fn ba_print(run_id: i64, msg: impl Into<String>) -> Self {
		Self::BaPrint(RunPrint {
			run_id,
			msg: msg.into(),
		})
	}
	pub fn ba_end(run_id: i64, time_us: i64) -> Self {
		Self::BaEnd(RunTs { run_id, time_us })
	}

	// -- Data
	pub fn dt_start(task_id: i64) -> Self {
		Self::DtStart { task_id }
	}
}

// region:    --- Sub Types

#[derive(Debug, Clone)]
pub struct RunTs {
	pub run_id: i64,
	pub time_us: i64,
}

#[derive(Debug, Clone)]
pub struct RunPrint {
	pub run_id: i64,
	pub msg: String,
}

#[derive(Debug, Clone)]
pub enum LuaEvent {
	FileSave { path: String },
	Print { message: String },
}

// endregion: --- Sub Types
