use crate::store::Result;
use crate::store::{Id, ModelManager, base};
use uuid::Uuid;

pub trait DbBmc: Sized {
	const TABLE: &'static str;

	fn table_ref() -> &'static str {
		Self::TABLE
	}

	fn get_uid(mm: &ModelManager, id: Id) -> Result<Uuid> {
		base::get_uid::<Self>(mm, id)
	}
}
