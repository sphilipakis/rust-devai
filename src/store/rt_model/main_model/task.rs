use crate::store::base::{self, DbBmc};
use crate::store::rt_model::{Inout, InoutBmc, InoutForCreate, InoutOnlyDisplay};
use crate::store::{EndState, Id, ModelManager, Result, RunningState, TypedContent, UnixTimeUs};
use crate::support::time::now_micro;
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields, SqliteField};
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

	// -- End state & Data
	pub end_state: Option<EndState>,
	pub end_err_id: Option<Id>,
	pub end_skip_reason: Option<String>,

	pub model_ov: Option<String>,

	// -- Usage values
	pub tk_prompt_total: Option<i64>,
	pub tk_prompt_cached: Option<i64>,
	pub tk_prompt_cache_creation: Option<i64>,
	pub tk_completion_total: Option<i64>,
	pub tk_completion_reasoning: Option<i64>,

	pub cost: Option<f64>,

	pub label: Option<String>,

	// -- Might want to have some input_short: Option<String>
	pub input_uid: Option<Uuid>,
	pub input_has_display: Option<bool>,

	pub output_uid: Option<Uuid>,
	pub output_has_display: Option<bool>,
}

impl Task {
	pub fn is_ended(&self) -> bool {
		matches!(RunningState::from(self), RunningState::Ended(_))
	}
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct TaskForCreate {
	pub run_id: Id,
	pub idx: i64,

	pub label: Option<String>,

	#[field(skip)]
	pub input_content: Option<TypedContent>,
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

	// -- End state & Data
	pub end_state: Option<EndState>,
	pub end_err_id: Option<Id>,
	pub end_skip_reason: Option<String>,

	// -- Usage values
	pub tk_prompt_total: Option<i32>,
	pub tk_prompt_cached: Option<i32>,
	pub tk_prompt_cache_creation: Option<i32>,
	pub tk_completion_total: Option<i32>,
	pub tk_completion_reasoning: Option<i32>,

	pub model_ov: Option<String>,
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

// region:    --- Froms

impl From<&Task> for RunningState {
	fn from(value: &Task) -> Self {
		if value.end.is_some() {
			RunningState::Ended(value.end_state)
		} else if value.start.is_some() {
			RunningState::Running
		} else {
			RunningState::Waiting
		}
	}
}

// endregion: --- Froms

// region:    --- Bmc

pub struct TaskBmc;

impl DbBmc for TaskBmc {
	const TABLE: &'static str = "task";
}

/// Basic CRUD
impl TaskBmc {
	pub fn create(mm: &ModelManager, mut task_c: TaskForCreate) -> Result<Id> {
		let input_content = task_c.input_content.take();

		// -- Add input_uid
		let mut task_fields = task_c.sqlite_not_none_fields();
		// add input_uid if present
		if let Some(input_uid) = input_content.as_ref().map(|v| v.uid) {
			task_fields.push(SqliteField::new("input_uid", input_uid));
		}
		if let Some(input_has_display) = input_content.as_ref().map(|v| v.display.is_some()) {
			task_fields.push(SqliteField::new("input_has_display", input_has_display));
		}

		// -- Add task
		let id = base::create::<Self>(mm, task_fields)?;

		// -- Add input Content
		if let Some(input_content) = input_content {
			let task_uid = TaskBmc::get_uid(mm, id)?;
			InoutBmc::create(
				mm,
				InoutForCreate {
					uid: input_content.uid,
					task_uid,
					typ: Some(input_content.typ),
					content: input_content.content,
					display: input_content.display,
				},
			)?;
		}

		Ok(id)
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
}

/// Task Specific bmcs
impl TaskBmc {
	/// List the task for a given run_id
	/// NOTE: Order id ASC (default)
	pub fn list_for_run(mm: &ModelManager, run_id: Id) -> Result<Vec<Task>> {
		let filter = TaskFilter { run_id: Some(run_id) };
		Self::list(mm, None, Some(filter))
	}

	pub fn end_with_error(mm: &ModelManager, task_id: Id, error: &crate::error::Error) -> Result<()> {
		use crate::store::ContentTyp;
		use crate::store::rt_model::{ErrBmc, ErrForCreate};

		let task = Self::get(mm, task_id)?;

		// -- Create the err rec
		let err_c = ErrForCreate {
			stage: None,
			run_id: Some(task.run_id),
			task_id: Some(task_id),
			typ: Some(ContentTyp::Text),
			content: Some(error.to_string()),
		};
		let err_id = ErrBmc::create(mm, err_c)?;

		// -- Update the task
		let task_u = TaskForUpdate {
			end: Some(now_micro().into()),
			end_state: Some(EndState::Err),
			end_err_id: Some(err_id),
			..Default::default()
		};
		Self::update(mm, task_id, task_u)?;

		Ok(())
	}

	/// return number of affected
	pub fn cancel_all_not_ended_for_run(mm: &ModelManager, run_id: Id) -> Result<usize> {
		let tasks_u = TaskForUpdate {
			end_state: Some(EndState::Cancel),
			end: Some(now_micro().into()), // NOTE this means sometime might have end without start
			..Default::default()
		};
		let table_name = Self::table_ref();

		let update_fields = tasks_u.sqlite_not_none_fields();

		let sql = format!(
			"UPDATE {table_name} SET {} where run_id = ? AND end_state IS NULL",
			update_fields.sql_setters()
		);

		let all_fields = update_fields.append(SqliteField::new("run_id", run_id));

		// -- Execute the command
		let values = all_fields.values_as_dyn_to_sql_vec();
		let db = mm.db();

		let num = db.exec(&sql, &*values)?;

		Ok(num)
	}
}

// endregion: --- Bmc

// region:    --- Bmc going to Content Model

impl TaskBmc {
	/// Note: Used by tui
	pub fn get_input_for_display(mm: &ModelManager, task: &Task) -> Result<Option<String>> {
		let input_has_display = task.input_has_display.unwrap_or_default();
		let Some(input_uid) = task.input_uid.as_ref() else {
			return Ok(None);
		};

		if input_has_display {
			// if not found, return None
			Ok(InoutBmc::get_by_uid::<InoutOnlyDisplay>(mm, *input_uid)
				.map(|i| i.display)
				.ok()
				.flatten())
		} else {
			Ok(InoutBmc::get_by_uid::<Inout>(mm, *input_uid).map(|i| i.content).ok().flatten())
		}
	}

	/// Note: Used by tui
	pub fn get_output_for_display(mm: &ModelManager, task: &Task) -> Result<Option<String>> {
		let output_has_display = task.output_has_display.unwrap_or_default();
		let Some(output_uid) = task.output_uid.as_ref() else {
			return Ok(None);
		};

		if output_has_display {
			// if not found, return None
			Ok(InoutBmc::get_by_uid::<InoutOnlyDisplay>(mm, *output_uid)
				.map(|i| i.display)
				.ok()
				.flatten())
		} else {
			Ok(InoutBmc::get_by_uid::<Inout>(mm, *output_uid).map(|i| i.content).ok().flatten())
		}
	}

	/// Note: used from runtime_rec
	pub fn update_output(mm: &ModelManager, task_id: Id, content: TypedContent) -> Result<()> {
		// -- Create the task fields
		// NOTE: Manual for now (not in common TaskForUpdate, might be TaskForUpdateContent later)
		let task_u_fields = vec![
			SqliteField::new("output_uid", content.uid),
			SqliteField::new("output_has_display", content.display.is_some()),
		];

		// -- Create the Inout
		let task_uid = Self::get_uid(mm, task_id)?;
		InoutBmc::create(
			mm,
			InoutForCreate {
				uid: content.uid,
				task_uid,
				typ: Some(content.typ),
				content: content.content,
				display: content.display,
			},
		)?;

		// -- Update the tasks
		base::update::<Self>(mm, task_id, task_u_fields.into())?;

		Ok(())
	}
}

// endregion: --- Bmc going to Content Model

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::store::rt_model::{RunBmc, RunForCreate};
	use crate::support::time::now_micro;
	use modql::filter::OrderBy;

	// region:    --- Support
	async fn create_run(mm: &ModelManager, label: &str) -> Result<Id> {
		let run_c = RunForCreate {
			agent_name: Some(label.to_string()),
			agent_path: Some(format!("path/{label}")),
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
			input_content: None,
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
			input_content: None,
		};
		let id = TaskBmc::create(&mm, task_c)?;

		// -- Exec
		let task_u = TaskForUpdate {
			start: Some(now_micro().into()),
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
				input_content: None,
			};
			TaskBmc::create(&mm, task_c)?;
		}

		// -- Exec
		let tasks: Vec<Task> = TaskBmc::list(&mm, Some(ListOptions::default()), None)?;

		// -- Check
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
				input_content: None,
			};
			TaskBmc::create(&mm, task_c)?;
		}

		// -- Exec
		let tasks: Vec<Task> = TaskBmc::list(&mm, Some(ListOptions::default()), None)?;

		// -- Check
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
				input_content: None,
			};
			TaskBmc::create(&mm, task_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let tasks: Vec<Task> = TaskBmc::list(&mm, Some(list_options), None)?;

		// -- Check
		assert_eq!(tasks.len(), 3);
		let task = tasks.first().ok_or("Should have first item")?;
		assert_eq!(task.id, 3.into());
		assert_eq!(task.label, Some("label-2".to_string()));
		let task = tasks.get(2).ok_or("Should have third item")?;
		assert_eq!(task.id, 1.into());
		assert_eq!(task.label, Some("label-0".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_task_cancel_all_not_ended_for_run() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let task_c = TaskForCreate {
				run_id,
				idx: 1 + 1,
				label: Some(format!("label-{i}")),
				input_content: None,
			};
			TaskBmc::create(&mm, task_c)?;
		}
		// We end the first one (yes, assume 1)
		TaskBmc::update(
			&mm,
			1.into(),
			TaskForUpdate {
				end: Some(now_micro().into()),
				end_state: Some(EndState::Ok),
				..Default::default()
			},
		)?;
		// helper fn
		let count_ends_fn = || -> Result<i32> {
			Ok(TaskBmc::list(&mm, None, Some(TaskFilter { run_id: Some(run_id) }))?
				.into_iter()
				.map(|t| t.end.map(|_| 1).unwrap_or_default())
				.sum::<i32>())
		};
		assert_eq!(count_ends_fn()?, 1);

		// -- Exec
		TaskBmc::cancel_all_not_ended_for_run(&mm, run_id)?;
		assert_eq!(count_ends_fn()?, 3); // how we should have 3 end
		// check end_state
		let states: Vec<EndState> = TaskBmc::list(&mm, None, Some(TaskFilter { run_id: Some(run_id) }))?
			.into_iter()
			.filter_map(|t| t.end_state)
			.collect();
		assert_eq!(&format!("{states:?}"), "[Ok, Cancel, Cancel]");

		Ok(())
	}
}

// endregion: --- Tests
