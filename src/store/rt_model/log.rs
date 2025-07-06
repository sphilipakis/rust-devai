use crate::derive_simple_enum_type;
use crate::store::base::{self, DbBmc};
use crate::store::{Id, ModelManager, Result, RunStep, Stage, UnixTimeUs};
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

	pub level: Option<LogLevel>,

	pub step: Option<RunStep>,

	pub stage: Option<Stage>,

	pub message: Option<String>,
}

derive_simple_enum_type! {
pub enum LogLevel {
	SysInfo,
	SysWarn,
	SysError,
	SysDebug,
	AgentPrint,
}
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct LogForCreate {
	pub run_id: Id,
	pub task_id: Option<Id>,

	pub level: Option<LogLevel>,

	pub step: Option<RunStep>,

	// The logical processing stage when the log entry is created.
	pub stage: Option<Stage>,

	pub message: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct LogForUpdate {
	pub level: Option<LogLevel>,

	// Optionally update the processing stage for this log entry.
	pub stage: Option<Stage>,

	pub message: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct LogFilter {
	pub run_id: Option<Id>,
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

	pub fn list(
		mm: &ModelManager,
		list_options: Option<ListOptions>,
		log_filter: Option<LogFilter>,
	) -> Result<Vec<Log>> {
		let filter_fields = log_filter.map(|f| f.sqlite_not_none_fields());
		base::list::<Self, _>(mm, list_options, filter_fields)
	}

	pub fn list_for_display(mm: &ModelManager, run_id: Id) -> Result<Vec<Log>> {
		let list_options = ListOptions::from_order_bys("!id");
		let filter = LogFilter { run_id: Some(run_id) };
		Self::list(mm, Some(list_options), Some(filter))
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
			level: Some(LogLevel::SysInfo),
			step: Some(RunStep::AaEnd),
			stage: Some(Stage::AfterAll),
			message: Some("First message".to_string()),
		};
		let id = LogBmc::create(&mm, log_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);
		let log: Log = LogBmc::get(&mm, id)?;
		assert_eq!(log.stage, Some(Stage::AfterAll));
		assert_eq!(log.step, Some(RunStep::AaEnd));
		assert_eq!(log.level, Some(LogLevel::SysInfo));

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
			level: None,
			stage: None,
			step: None,
			message: Some("Before update".to_string()),
		};
		let id = LogBmc::create(&mm, log_c)?;

		// -- Exec
		let log_u = LogForUpdate {
			message: Some(format!("Updated at {}", now_unix_time_us())),
			level: Some(LogLevel::SysWarn),
			..Default::default()
		};
		LogBmc::update(&mm, id, log_u)?;

		// -- Check
		let log = LogBmc::get(&mm, id)?;
		assert!(log.message.unwrap().starts_with("Updated"));
		assert_eq!(log.level, Some(LogLevel::SysWarn));

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
				level: None,
				stage: None,
				step: None,
				message: Some(format!("msg-{i}")),
			};
			LogBmc::create(&mm, log_c)?;
		}

		// -- Exec
		let logs: Vec<Log> = LogBmc::list(&mm, Some(ListOptions::default()), None)?;

		// -- Check
		assert_eq!(logs.len(), 3);
		let log = logs.first().ok_or("Should have first item")?;
		assert_eq!(log.id, 1.into());
		assert_eq!(log.message, Some("msg-0".to_string()));
		assert!(log.level.is_none());

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
				level: if i == 2 { Some(LogLevel::SysDebug) } else { None },
				stage: None,
				step: None,
				message: Some(format!("msg-{i}")),
			};
			LogBmc::create(&mm, log_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let logs: Vec<Log> = LogBmc::list(&mm, Some(list_options), None)?;

		// -- Check
		assert_eq!(logs.len(), 3);
		let log = logs.first().ok_or("Should have first item")?;
		assert_eq!(log.id, 3.into());
		assert_eq!(log.message, Some("msg-2".to_string()));
		assert_eq!(log.level, Some(LogLevel::SysDebug));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_log_bmc_list_with_filter() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_1_id = create_run(&mm, "run-1").await?;
		let run_2_id = create_run(&mm, "run-2").await?;
		for run_id in [run_1_id, run_2_id] {
			for i in 0..3 {
				let log_c = LogForCreate {
					run_id,
					task_id: None,
					level: if i == 2 { Some(LogLevel::SysDebug) } else { None },
					stage: None,
					step: None,
					message: Some(format!("msg-{i}")),
				};
				LogBmc::create(&mm, log_c)?;
			}
		}

		// -- Exec
		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);
		let filter = LogFilter { run_id: Some(run_1_id) };
		let logs: Vec<Log> = LogBmc::list(&mm, Some(list_options), Some(filter))?;

		// -- Check
		assert_eq!(logs.len(), 3);
		let log = logs.first().ok_or("Should have first item")?;
		assert_eq!(log.id, 3.into());
		assert_eq!(log.message, Some("msg-2".to_string()));
		assert_eq!(log.level, Some(LogLevel::SysDebug));

		Ok(())
	}
}

// endregion: --- Tests
