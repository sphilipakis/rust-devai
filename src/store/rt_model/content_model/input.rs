use crate::store::base::{self, DbBmc};
use crate::store::{ContentTyp, Id, ModelManager, Result, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Input {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub task_uid: Uuid,

	pub typ: Option<String>,
	pub content: Option<String>,

	pub display: Option<String>,
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct InputOnlyDisplay {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub task_uid: Uuid,

	pub display: Option<String>,
}

pub trait InputRecord {}
impl InputRecord for Input {}
impl InputRecord for InputOnlyDisplay {}

// NOTE: Content table have uid in the ForCreate (as they are pre-linked to main)
#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct InputForCreate {
	pub uid: Uuid,
	pub task_uid: Uuid,

	pub typ: Option<ContentTyp>,
	pub content: Option<String>,

	pub display: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct InputForUpdate {
	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct InputFilter {
	pub task_uid: Option<Uuid>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct InputBmc;

impl DbBmc for InputBmc {
	const TABLE: &'static str = "input";
}

impl InputBmc {
	pub fn create(mm: &ModelManager, input_c: InputForCreate) -> Result<Id> {
		let fields = input_c.sqlite_not_none_fields();
		base::create_uid_included::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, input_u: InputForUpdate) -> Result<usize> {
		let fields = input_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	pub fn get<REC>(mm: &ModelManager, id: Id) -> Result<REC>
	where
		REC: HasSqliteFields + SqliteFromRow + Unpin + Send,
	{
		base::get::<Self, REC>(mm, id)
	}

	pub fn get_by_uid<REC>(mm: &ModelManager, uid: Uuid) -> Result<REC>
	where
		REC: HasSqliteFields + SqliteFromRow + Unpin + Send,
	{
		base::get_by_uid::<Self, REC>(mm, uid)
	}

	pub fn list(
		mm: &ModelManager,
		list_options: Option<ListOptions>,
		filter: Option<InputFilter>,
	) -> Result<Vec<Input>> {
		let filter_fields = filter.map(|f| f.sqlite_not_none_fields());
		base::list::<Self, _>(mm, list_options, filter_fields)
	}

	/// Convenience helper to list all inputs for a given task `uid`.
	pub fn first_for_task(mm: &ModelManager, task_uid: Uuid) -> Result<Option<Input>> {
		let filter = InputFilter {
			task_uid: Some(task_uid),
		};
		base::first::<Self, _>(mm, None, Some(filter.sqlite_not_none_fields()))
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
	async fn create_run_and_task(mm: &ModelManager) -> Result<(Uuid, Uuid)> {
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
				label: Some("task".into()),
				input_content: None,
			},
		)?;
		let task = TaskBmc::get(mm, task_id)?;
		Ok((task.uid, Uuid::new_v4()))
	}
	// endregion: --- Support
}

// endregion: --- Tests
