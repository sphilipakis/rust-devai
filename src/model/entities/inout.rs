use crate::model::ScalarEnumType;
use crate::model::base::{self, DbBmc};
use crate::model::{ContentTyp, Id, ModelManager, Result, UnixTimeUs};
use macro_rules_attribute as mra;
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Inout {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub task_uid: Uuid,

	pub kind: Option<InoutKind>,

	pub typ: Option<String>,
	pub content: Option<String>,

	pub display: Option<String>,
}

#[mra::derive(Debug, ScalarEnumType!)]
pub enum InoutKind {
	In,
	Out,
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct InoutOnlyDisplay {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub task_uid: Uuid,

	pub display: Option<String>,
}

pub trait InoutRecord {}
impl InoutRecord for Inout {}
impl InoutRecord for InoutOnlyDisplay {}

// NOTE: Content table have uid in the ForCreate (as they are pre-linked to main)
#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct InoutForCreate {
	pub uid: Uuid,
	pub task_uid: Uuid,

	pub typ: Option<ContentTyp>,
	pub content: Option<String>,

	pub display: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct InoutForUpdate {
	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct InoutFilter {
	pub task_uid: Option<Uuid>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct InoutBmc;

impl DbBmc for InoutBmc {
	const TABLE: &'static str = "inout";
}

impl InoutBmc {
	pub fn create(mm: &ModelManager, input_c: InoutForCreate) -> Result<Id> {
		let fields = input_c.sqlite_not_none_fields();
		base::create_uid_included::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, input_u: InoutForUpdate) -> Result<usize> {
		let fields = input_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	#[allow(unused)]
	pub fn get<REC>(mm: &ModelManager, id: Id) -> Result<REC>
	where
		REC: HasSqliteFields + SqliteFromRow + Unpin + Send,
		REC: InoutRecord,
	{
		base::get::<Self, REC>(mm, id)
	}

	pub fn get_by_uid<REC>(mm: &ModelManager, uid: Uuid) -> Result<REC>
	where
		REC: HasSqliteFields + SqliteFromRow + Unpin + Send,
		REC: InoutRecord,
	{
		base::get_by_uid::<Self, REC>(mm, uid)
	}

	#[allow(unused)]
	pub fn list(
		mm: &ModelManager,
		list_options: Option<ListOptions>,
		filter: Option<InoutFilter>,
	) -> Result<Vec<Inout>> {
		let filter_fields = filter.map(|f| f.sqlite_not_none_fields());
		base::list::<Self, _>(mm, list_options, filter_fields)
	}
}

// endregion: --- Bmc
