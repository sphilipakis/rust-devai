use crate::store::base::{self, DbBmc};
use crate::store::{Id, ModelManager, Result, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Output {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub task_uid: Uuid,

	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct OutputForCreate {
	pub task_uid: Uuid,

	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct OutputForUpdate {
	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct OutputFilter {
	pub task_uid: Option<Uuid>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct OutputBmc;

impl DbBmc for OutputBmc {
	const TABLE: &'static str = "output";
}

impl OutputBmc {
	pub fn create(mm: &ModelManager, output_c: OutputForCreate) -> Result<Id> {
		let fields = output_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, output_u: OutputForUpdate) -> Result<usize> {
		let fields = output_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	#[allow(unused)]
	pub fn get(mm: &ModelManager, id: Id) -> Result<Output> {
		base::get::<Self, _>(mm, id)
	}

	pub fn list(
		mm: &ModelManager,
		list_options: Option<ListOptions>,
		filter: Option<OutputFilter>,
	) -> Result<Vec<Output>> {
		let filter_fields = filter.map(|f| f.sqlite_not_none_fields());
		base::list::<Self, _>(mm, list_options, filter_fields)
	}

	pub fn list_for_task(mm: &ModelManager, task_uid: Uuid) -> Result<Vec<Output>> {
		let filter = OutputFilter {
			task_uid: Some(task_uid),
		};
		Self::list(mm, None, Some(filter))
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::store::rt_model::{RunBmc, RunForCreate, TaskBmc, TaskForCreate};
	use uuid::Uuid;

	// region:    --- Support
	async fn create_run_and_task(mm: &ModelManager) -> Result<Uuid> {
		let run_id = RunBmc::create(
			mm,
			RunForCreate {
				agent_name: Some("run".into()),
				agent_path: Some("path/run".into()),
				start: None,
			},
		)?;
		let task_id = TaskBmc::create(
			mm,
			TaskForCreate {
				run_id,
				idx: 1,
				input_content: None,
				label: Some("task".into()),
			},
		)?;
		Ok(TaskBmc::get(mm, task_id)?.uid)
	}
	// endregion: --- Support

	#[tokio::test]
	async fn test_model_output_bmc_create_and_get() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let task_uid = create_run_and_task(&mm).await?;

		// -- Exec
		let output_c = OutputForCreate {
			task_uid,
			typ: Some("json".into()),
			content: Some(r#"{"a":1}"#.into()),
		};
		let id = OutputBmc::create(&mm, output_c)?;

		// -- Check
		let output = OutputBmc::get(&mm, id)?;
		assert_eq!(output.typ, Some("json".to_string()));
		assert_eq!(output.content, Some(r#"{"a":1}"#.to_string()));

		Ok(())
	}
}

// endregion: --- Tests
