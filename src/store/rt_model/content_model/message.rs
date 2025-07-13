use crate::store::base::{self, DbBmc};
use crate::store::{Id, ModelManager, Result, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Message {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub task_uid: Uuid,

	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct MessageForCreate {
	pub task_uid: Uuid,

	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct MessageForUpdate {
	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct MessageFilter {
	pub task_uid: Option<Uuid>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct MessageBmc;

impl DbBmc for MessageBmc {
	const TABLE: &'static str = "message";
}

#[allow(unused)]
impl MessageBmc {
	pub fn create(mm: &ModelManager, msg_c: MessageForCreate) -> Result<Id> {
		let fields = msg_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, msg_u: MessageForUpdate) -> Result<usize> {
		let fields = msg_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	#[allow(unused)]
	pub fn get(mm: &ModelManager, id: Id) -> Result<Message> {
		base::get::<Self, _>(mm, id)
	}

	pub fn list(
		mm: &ModelManager,
		list_options: Option<ListOptions>,
		filter: Option<MessageFilter>,
	) -> Result<Vec<Message>> {
		let filter_fields = filter.map(|f| f.sqlite_not_none_fields());
		base::list::<Self, _>(mm, list_options, filter_fields)
	}

	pub fn list_for_task(mm: &ModelManager, task_uid: Uuid) -> Result<Vec<Message>> {
		let filter = MessageFilter {
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
				label: Some("task".into()),
				input_content: None,
			},
		)?;
		Ok(TaskBmc::get(mm, task_id)?.uid)
	}
	// endregion: --- Support

	#[tokio::test]
	async fn test_model_message_bmc_create_and_list() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let task_uid = create_run_and_task(&mm).await?;

		// -- Exec
		for i in 0..2 {
			let msg_c = MessageForCreate {
				task_uid,
				typ: Some("text".into()),
				content: Some(format!("msg-{i}")),
			};
			MessageBmc::create(&mm, msg_c)?;
		}

		// -- Check
		let msgs = MessageBmc::list_for_task(&mm, task_uid)?;
		assert_eq!(msgs.len(), 2);
		assert_eq!(msgs[1].content, Some("msg-1".to_string()));

		Ok(())
	}
}

// endregion: --- Tests
