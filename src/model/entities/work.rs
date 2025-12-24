use crate::model::ScalarEnum;
use crate::model::base::{self, DbBmc};
use crate::model::{EndState, EpochUs, Error, Id, ModelManager, Result, RunningState};
use macro_rules_attribute as mra;
use modql::SqliteFromRow;
use modql::field::{Fields, HasFields as _, HasSqliteFields};
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Work {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: EpochUs,
	pub mtime: EpochUs,

	pub kind: WorkKind,

	pub start: Option<EpochUs>,
	pub end: Option<EpochUs>,

	pub end_state: Option<EndState>,
	pub end_err_id: Option<Id>,

	pub data: Option<String>, // JSON
	pub message: Option<String>,
}

impl Work {
	pub fn get_data_as<T: serde::de::DeserializeOwned>(&self) -> Result<Option<T>> {
		let Some(data) = &self.data else {
			return Ok(None);
		};
		let val = serde_json::from_str(data).map_err(|err| Error::cc("Cannot dserialize WorkData", err))?;
		Ok(Some(val))
	}
}

#[mra::derive(Debug, ScalarEnum!)]
pub enum WorkKind {
	Install,
}

impl From<&Work> for RunningState {
	fn from(value: &Work) -> Self {
		if value.end.is_some() {
			RunningState::Ended(value.end_state)
		} else if value.start.is_some() {
			RunningState::Running
		} else {
			RunningState::Waiting
		}
	}
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct WorkForCreate {
	pub kind: WorkKind,
	pub data: Option<String>,
}

impl WorkForCreate {
	pub fn set_data<T: serde::Serialize>(&mut self, data: &T) -> Result<()> {
		let json = serde_json::to_string(data).map_err(|err| Error::cc("Cannot serialize WorkData", err))?;
		self.data = Some(json);
		Ok(())
	}
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct WorkForUpdate {
	pub start: Option<EpochUs>,
	pub end: Option<EpochUs>,

	pub end_state: Option<EndState>,
	pub end_err_id: Option<Id>,

	pub data: Option<String>,
	pub message: Option<String>,
}

impl WorkForUpdate {
	pub fn set_data<T: serde::Serialize>(&mut self, data: &T) -> Result<()> {
		let json = serde_json::to_string(data).map_err(|err| Error::cc("Cannot serialize WorkData", err))?;
		self.data = Some(json);
		Ok(())
	}
}

// endregion: --- Types

// region:    --- Bmc

pub struct WorkBmc;

impl DbBmc for WorkBmc {
	const TABLE: &'static str = "work";
}

impl WorkBmc {
	pub fn create(mm: &ModelManager, work_c: WorkForCreate) -> Result<Id> {
		let fields = work_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	pub fn update(mm: &ModelManager, id: Id, work_u: WorkForUpdate) -> Result<usize> {
		let fields = work_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	pub fn get(mm: &ModelManager, id: Id) -> Result<Work> {
		base::get::<Self, _>(mm, id)
	}
}

/// WorkBmc for Install kind
impl WorkBmc {
	pub fn get_active_install(mm: &ModelManager) -> Result<Option<Work>> {
		let sql = format!(
			"SELECT {} FROM {} WHERE kind = 'Install' AND end IS NULL ORDER BY id DESC LIMIT 1",
			Work::sql_columns(),
			Self::table_ref()
		);
		let db = mm.db();
		let entity = db.fetch_first(&sql, ())?;
		Ok(entity)
	}

	#[allow(unused)]
	pub fn get_latest_install_for_pack(mm: &ModelManager, pack_ref: &str) -> Result<Option<Work>> {
		let sql = format!(
			"SELECT {} FROM {} WHERE kind = 'Install' AND json_extract(data, '$.pack_ref') = ? ORDER BY id DESC LIMIT 1",
			Work::sql_columns(),
			Self::table_ref()
		);
		let db = mm.db();
		let entity = db.fetch_first(&sql, (pack_ref,))?;
		Ok(entity)
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[tokio::test]
	async fn test_model_work_bmc_create() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;

		// -- Exec
		let work_c = WorkForCreate {
			kind: WorkKind::Install,
			data: Some(r#"{"pack_ref": "some@pack"}"#.to_string()),
		};
		let id = WorkBmc::create(&mm, work_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);
		let work = WorkBmc::get(&mm, id)?;
		assert_eq!(work.kind, WorkKind::Install);
		assert!(work.data.is_some());
		assert!(work.start.is_none());

		Ok(())
	}
}

// endregion: --- Tests
