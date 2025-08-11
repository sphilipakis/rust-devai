use crate::store::base::{self, DbBmc};
use crate::store::{ContentTyp, Id, ModelManager, Result, Stage, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasFields as _, HasSqliteFields};
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct ErrRec {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub stage: Option<Stage>,

	// Foreign keys (optional – allow global errors)
	pub run_id: Option<Id>,
	pub task_id: Option<Id>,

	pub typ: Option<String>,
	pub content: Option<String>,
}

/// Same fields as the main table but without the IDs/ctime/mtime.
/// Note: `typ` uses `ContentTyp` for stronger typing on create.
#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct ErrForCreate {
	pub stage: Option<Stage>,

	pub run_id: Option<Id>,
	pub task_id: Option<Id>,

	pub typ: Option<ContentTyp>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct ErrForUpdate {
	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
#[allow(unused)]
pub struct ErrFilter {
	pub run_id: Option<Id>,
	pub task_id: Option<Id>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct ErrBmc;

impl DbBmc for ErrBmc {
	const TABLE: &'static str = "err";
}

impl ErrBmc {
	pub fn create(mm: &ModelManager, err_c: ErrForCreate) -> Result<Id> {
		let fields = err_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, err_u: ErrForUpdate) -> Result<usize> {
		let fields = err_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	pub fn get(mm: &ModelManager, id: Id) -> Result<ErrRec> {
		base::get::<Self, _>(mm, id)
	}

	/// Used by the TUI to get system err (not from run or task)
	pub fn first_system_err(mm: &ModelManager) -> Result<Option<ErrRec>> {
		let sql = format!(
			"SELECT {} FROM {} WHERE run_id IS NULL AND task_id IS NULL ORDER BY id  LIMIT 1 ",
			ErrRec::sql_columns(),
			Self::table_ref(),
		);

		// -- Exec query
		let db = mm.db();
		let entities: Vec<ErrRec> = db.fetch_all(&sql, ())?;

		Ok(entities.into_iter().next())
	}
}

// endregion: --- Bmc

// region:    --- Tests

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::store::rt_model::{RunBmc, RunForCreate, TaskBmc, TaskForCreate};
	use crate::support::time::now_micro;

	// region:    --- Support

	async fn create_run(mm: &ModelManager, label: &str) -> Result<Id> {
		let run_c = RunForCreate {
			parent_id: None,
			agent_name: Some(label.to_string()),
			agent_path: Some(format!("path/{label}")),
			has_task_stages: None,
			has_prompt_parts: None,
		};
		Ok(RunBmc::create(mm, run_c)?)
	}

	async fn create_task(mm: &ModelManager, run_id: Id, idx: i64) -> Result<Id> {
		let task_c = TaskForCreate {
			run_id,
			idx,
			label: Some(format!("task-{idx}")),
			input_content: None,
		};
		Ok(TaskBmc::create(mm, task_c)?)
	}

	// endregion: --- Support

	#[tokio::test]
	async fn test_model_err_bmc_create() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let task_id = create_task(&mm, run_id, 1).await?;

		// -- Exec
		let err_c = ErrForCreate {
			stage: None,
			run_id: Some(run_id),
			task_id: Some(task_id),
			typ: Some(ContentTyp::Text),
			content: Some("Something went wrong".to_string()),
		};
		let id = ErrBmc::create(&mm, err_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);
		let err_rec = ErrBmc::get(&mm, id)?;
		assert_eq!(err_rec.run_id, Some(run_id));
		assert_eq!(err_rec.task_id, Some(task_id));
		assert_eq!(err_rec.typ, Some("Text".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_err_bmc_update() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let err_c = ErrForCreate {
			stage: None,
			run_id: Some(run_id),
			task_id: None,
			typ: Some(ContentTyp::Text),
			content: Some("Before update".to_string()),
		};
		let id = ErrBmc::create(&mm, err_c)?;

		// -- Exec
		let err_u = ErrForUpdate {
			content: Some(format!("Updated at {}", now_micro())),
			..Default::default()
		};
		ErrBmc::update(&mm, id, err_u)?;

		// -- Check
		let err_rec = ErrBmc::get(&mm, id)?;
		assert!(err_rec.content.unwrap().starts_with("Updated"));

		Ok(())
	}
}

// endregion: --- Tests
