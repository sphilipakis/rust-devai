use crate::derive_simple_enum_type;
use crate::store::base::{self, DbBmc};
use crate::store::{Id, ModelManager, Result, Stage, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Log {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	// Foreign keys
	pub run_id: Id,
	pub task_id: Option<Id>,

	pub kind: Option<LogKind>,

	pub stage: Option<Stage>,

	pub message: Option<String>,
}

derive_simple_enum_type! {
pub enum LogKind {
	SysInfo,
	SysWarn,
	SysDebug,
}
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct LogForCreate {
	pub run_id: Id,
	pub task_id: Option<Id>,

	pub kind: Option<LogKind>,

	// The logical processing stage when the log entry is created.
	pub stage: Option<Stage>,

	pub message: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct LogForUpdate {
	pub kind: Option<LogKind>,

	// Optionally update the processing stage for this log entry.
	pub stage: Option<Stage>,

	pub message: Option<String>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct LogBmc;

impl DbBmc for LogBmc {
	const TABLE: &'static str = "log";
}

impl LogBmc {
	#[allow(unused)]
	pub fn create(mm: &ModelManager, log_c: LogForCreate) -> Result<Id> {
		let fields = log_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, log_u: LogForUpdate) -> Result<usize> {
		let fields = log_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	#[allow(unused)]
	pub fn get(mm: &ModelManager, id: Id) -> Result<Log> {
		base::get::<Self, _>(mm, id)
	}

	#[allow(unused)]
	pub fn list(mm: &ModelManager, list_options: Option<ListOptions>) -> Result<Vec<Log>> {
		base::list::<Self, _>(mm, list_options)
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::store::rt_model::{RunBmc, RunForCreate, TaskBmc, TaskForCreate};
	use crate::support::time::now_unix_time_us;
	use modql::filter::OrderBy;

	// region:    --- Support
	async fn create_run(mm: &ModelManager, label: &str) -> Result<Id> {
		let run_c = RunForCreate {
			agent_name: Some(label.to_string()),
			agent_path: Some(format!("path/{label}")),
			start: None,
		};
		Ok(RunBmc::create(mm, run_c)?)
	}

	async fn create_task(mm: &ModelManager, run_id: Id, num: i64) -> Result<Id> {
		let task_c = TaskForCreate {
			run_id,
			num: Some(num),
			label: Some(format!("task-{num}")),
		};
		Ok(TaskBmc::create(mm, task_c)?)
	}
	// endregion: --- Support

	#[tokio::test]
	async fn test_model_log_bmc_create() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let task_id = create_task(&mm, run_id, 1).await?;

		// -- Exec
		let log_c = LogForCreate {
			run_id,
			task_id: Some(task_id),
			kind: Some(LogKind::SysInfo),
			stage: Some(Stage::AfterAll),
			message: Some("First message".to_string()),
		};
		let id = LogBmc::create(&mm, log_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);
		let log: Log = LogBmc::get(&mm, id)?;
		assert_eq!(log.stage, Some(Stage::AfterAll));
		assert_eq!(log.kind, Some(LogKind::SysInfo));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_log_bmc_update() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let log_c = LogForCreate {
			run_id,
			task_id: None,
			kind: None,
			stage: None,
			message: Some("Before update".to_string()),
		};
		let id = LogBmc::create(&mm, log_c)?;

		// -- Exec
		let log_u = LogForUpdate {
			message: Some(format!("Updated at {}", now_unix_time_us())),
			kind: Some(LogKind::SysWarn),
			..Default::default()
		};
		LogBmc::update(&mm, id, log_u)?;

		// -- Check
		let log = LogBmc::get(&mm, id)?;
		assert!(log.message.unwrap().starts_with("Updated"));
		assert_eq!(log.kind, Some(LogKind::SysWarn));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_log_bmc_list_simple() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let log_c = LogForCreate {
				run_id,
				task_id: None,
				kind: None,
				stage: None,
				message: Some(format!("msg-{i}")),
			};
			LogBmc::create(&mm, log_c)?;
		}

		// -- Exec
		let logs: Vec<Log> = LogBmc::list(&mm, Some(ListOptions::default()))?;

		// -- Check
		assert_eq!(logs.len(), 3);
		let log = logs.first().ok_or("Should have first item")?;
		assert_eq!(log.id, 1.into());
		assert_eq!(log.message, Some("msg-0".to_string()));
		assert!(log.kind.is_none());

		Ok(())
	}

	#[tokio::test]
	async fn test_model_log_bmc_list_order_by() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let log_c = LogForCreate {
				run_id,
				task_id: None,
				kind: if i == 2 { Some(LogKind::SysDebug) } else { None },
				stage: None,
				message: Some(format!("msg-{i}")),
			};
			LogBmc::create(&mm, log_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let logs: Vec<Log> = LogBmc::list(&mm, Some(list_options))?;

		// -- Check
		assert_eq!(logs.len(), 3);
		let log = logs.first().ok_or("Should have first item")?;
		assert_eq!(log.id, 3.into());
		assert_eq!(log.message, Some("msg-2".to_string()));
		assert_eq!(log.kind, Some(LogKind::SysDebug));

		Ok(())
	}
}

// endregion: --- Tests
