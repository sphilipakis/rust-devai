use crate::store::base::{self, DbBmc};
use crate::store::{Id, ModelManager, Result, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Task {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	// Foreign key
	pub run_id: Id,

	pub idx: Option<i64>,

	// Step - Timestamps
	pub start: Option<UnixTimeUs>,
	pub data_start: Option<UnixTimeUs>,
	pub data_end: Option<UnixTimeUs>,
	pub ai_start: Option<UnixTimeUs>,
	pub ai_end: Option<UnixTimeUs>,
	pub output_start: Option<UnixTimeUs>,
	pub output_end: Option<UnixTimeUs>,
	pub end: Option<UnixTimeUs>,

	pub model: Option<String>,

	// -- Usage values
	pub tk_prompt_total: Option<i64>,
	pub tk_prompt_cached: Option<i64>,
	pub tk_prompt_cache_creation: Option<i64>,
	pub tk_completion_total: Option<i64>,
	pub tk_completion_reasoning: Option<i64>,

	pub cost: Option<f64>,

	pub label: Option<String>,
}

pub enum TaskState {
	Waiting,
	Running,
	Done,
}

impl Task {
	pub fn state(&self) -> TaskState {
		if self.end.is_some() {
			TaskState::Done
		} else if self.start.is_some() {
			TaskState::Running
		} else {
			TaskState::Waiting
		}
	}

	pub fn is_done(&self) -> bool {
		matches!(self.state(), TaskState::Done)
	}
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct TaskForCreate {
	pub run_id: Id,
	pub idx: i64,
	pub label: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct TaskForUpdate {
	// -- Step Timestamps
	pub start: Option<UnixTimeUs>,
	pub data_start: Option<UnixTimeUs>,
	pub data_end: Option<UnixTimeUs>,
	pub ai_start: Option<UnixTimeUs>,
	pub ai_end: Option<UnixTimeUs>,
	pub output_start: Option<UnixTimeUs>,
	pub output_end: Option<UnixTimeUs>,
	pub end: Option<UnixTimeUs>,

	pub model: Option<String>,

	// -- Usage values
	pub tk_prompt_total: Option<i32>,
	pub tk_prompt_cached: Option<i32>,
	pub tk_prompt_cache_creation: Option<i32>,
	pub tk_completion_total: Option<i32>,
	pub tk_completion_reasoning: Option<i32>,

	pub cost: Option<f64>,

	pub label: Option<String>,
}

impl TaskForUpdate {
	pub fn from_usage(usage: &genai::chat::Usage) -> Self {
		let tk_prompt_total = usage.prompt_tokens;
		let tk_prompt_cached = usage.prompt_tokens_details.as_ref().and_then(|d| d.cached_tokens);
		let tk_prompt_cache_creation = usage.prompt_tokens_details.as_ref().and_then(|d| d.cache_creation_tokens);
		let tk_completion_total = usage.completion_tokens;
		let tk_completion_reasoning = usage.completion_tokens_details.as_ref().and_then(|d| d.reasoning_tokens);

		Self {
			tk_prompt_total,
			tk_prompt_cached,
			tk_prompt_cache_creation,
			tk_completion_total,
			tk_completion_reasoning,
			..Default::default()
		}
	}
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct TaskFilter {
	pub run_id: Option<Id>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct TaskBmc;

impl DbBmc for TaskBmc {
	const TABLE: &'static str = "task";
}

impl TaskBmc {
	pub fn create(mm: &ModelManager, task_c: TaskForCreate) -> Result<Id> {
		let fields = task_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	pub fn update(mm: &ModelManager, id: Id, task_u: TaskForUpdate) -> Result<usize> {
		let fields = task_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	#[allow(unused)]
	pub fn get(mm: &ModelManager, id: Id) -> Result<Task> {
		base::get::<Self, _>(mm, id)
	}

	pub fn list(mm: &ModelManager, list_options: Option<ListOptions>, filter: Option<TaskFilter>) -> Result<Vec<Task>> {
		let filter_fields = filter.map(|f| f.sqlite_not_none_fields());
		base::list::<Self, _>(mm, list_options, filter_fields)
	}

	/// List the task for a given run_id
	/// NOTE: Order id ASC (default)
	pub fn list_for_run(mm: &ModelManager, run_id: Id) -> Result<Vec<Task>> {
		let filter = TaskFilter { run_id: Some(run_id) };
		Self::list(mm, None, Some(filter))
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::store::rt_model::{RunBmc, RunForCreate};
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
	// endregion: --- Support

	#[tokio::test]
	async fn test_model_task_bmc_create() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let task_c = TaskForCreate {
			run_id,
			idx: 1,
			label: Some("Test Task".to_string()),
		};

		// -- Exec
		let id = TaskBmc::create(&mm, task_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);

		Ok(())
	}

	#[tokio::test]
	async fn test_model_task_bmc_update_simple() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let task_c = TaskForCreate {
			run_id,
			idx: 1,
			label: Some("Test Task".to_string()),
		};
		let id = TaskBmc::create(&mm, task_c)?;

		// -- Exec
		let task_u = TaskForUpdate {
			start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		TaskBmc::update(&mm, id, task_u)?;

		// -- Check
		let task = TaskBmc::get(&mm, id)?;
		assert!(task.start.is_some());

		Ok(())
	}

	#[tokio::test]
	async fn test_model_task_bmc_list_simple() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let task_c = TaskForCreate {
				run_id,
				idx: 1 + 1,
				label: Some(format!("label-{i}")),
			};
			TaskBmc::create(&mm, task_c)?;
		}

		// -- Exec
		let tasks: Vec<Task> = TaskBmc::list(&mm, Some(ListOptions::default()), None)?;
		assert_eq!(tasks.len(), 3);
		let task = tasks.first().ok_or("Should have first item")?;
		assert_eq!(task.id, 1.into());
		assert_eq!(task.label, Some("label-0".to_string()));
		let task = tasks.get(2).ok_or("Should have 3 items")?;
		assert_eq!(task.id, 3.into());
		assert_eq!(task.label, Some("label-2".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_task_bmc_list_from_seed() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-seed").await?;
		for i in 0..10 {
			let task_c = TaskForCreate {
				run_id,
				idx: i + 1,
				label: Some(format!("label-{i}")),
			};
			TaskBmc::create(&mm, task_c)?;
		}

		// -- Exec
		let tasks: Vec<Task> = TaskBmc::list(&mm, Some(ListOptions::default()), None)?;
		assert_eq!(tasks.len(), 10);
		let task = tasks.first().ok_or("Should have first item")?;
		assert_eq!(task.id, 1.into());
		assert_eq!(task.label, Some("label-0".to_string()));
		let task = tasks.get(2).ok_or("Should have 3 items")?;
		assert_eq!(task.id, 3.into());
		assert_eq!(task.label, Some("label-2".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_task_bmc_list_order_by() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let task_c = TaskForCreate {
				run_id,
				idx: i + 1,
				label: Some(format!("label-{i}")),
			};
			TaskBmc::create(&mm, task_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let tasks: Vec<Task> = TaskBmc::list(&mm, Some(list_options), None)?;
		assert_eq!(tasks.len(), 3);
		let task = tasks.first().ok_or("Should have first item")?;
		assert_eq!(task.id, 3.into());
		assert_eq!(task.label, Some("label-2".to_string()));
		let task = tasks.get(2).ok_or("Should have third item")?;
		assert_eq!(task.id, 1.into());
		assert_eq!(task.label, Some("label-0".to_string()));

		Ok(())
	}
}

// endregion: --- Tests
