use crate::model::base::{self, DbBmc};
use crate::model::{Id, ModelManager, Result};
use crate::support::text;
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields as _};

// region:    --- Public Types

// NOTE: Not needed
// #[derive(Debug, Clone, Fields, SqliteFromRow)]
// pub struct Ucontent {
// 	pub id: Id,
// 	pub uid: Uuid,

// 	pub ctime: EpochUs,
// 	pub mtime: EpochUs,

// 	pub hash: String,
// 	pub is_json: bool,
// 	pub content: Option<String>,
// }

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct UcontentForCreate {
	pub hash: String,
	pub is_json: bool,
	pub content: Option<String>,
}

// endregion: --- Public Types

// region:    --- Bmc

pub struct UcontentBmc;

impl DbBmc for UcontentBmc {
	const TABLE: &'static str = "ucontent";
}

impl UcontentBmc {
	pub fn get_or_create_for_text(mm: &ModelManager, content: &str, is_json: bool) -> Result<Id> {
		let is_json_part = if is_json { "is_json=true" } else { "is_json=false" };
		let length_part = format!("length={}", content.len());
		let hash = text::blake3_b64u(&[is_json_part, &length_part, content]);

		if let Some(id) = Self::get_id_by_hash(mm, &hash)? {
			return Ok(id);
		}

		let ucontent_c = UcontentForCreate {
			hash,
			is_json,
			content: Some(content.to_string()),
		};

		Self::create(mm, ucontent_c)
	}
}

/// Private
impl UcontentBmc {
	fn create(mm: &ModelManager, ucontent_c: UcontentForCreate) -> Result<Id> {
		let fields = ucontent_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	pub fn get_id_by_hash(mm: &ModelManager, hash: &str) -> Result<Option<Id>> {
		let sql = "SELECT id FROM ucontent WHERE hash = ? LIMIT 1";
		let id = mm.db().exec_returning_as_optional::<i64>(sql, (hash,))?;

		Ok(id.map(|id| id.into()))
	}
}

// endregion: --- Bmc
