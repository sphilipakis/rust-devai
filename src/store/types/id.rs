use crate::store::ScalarStructType;
use crate::store::{Error, Result};
use macro_rules_attribute as mra;

// Simple wrapper for SQLite Ids
#[mra::derive(Debug, ScalarStructType!)]
pub struct Id(i64);

#[mra::derive(ScalarStructType!)]
pub struct Id2(i64);

impl Id {
	pub fn as_i64(&self) -> i64 {
		self.0
	}
}

// from &i64
impl From<&i64> for Id {
	fn from(val: &i64) -> Id {
		Id(*val)
	}
}

impl TryFrom<String> for Id {
	type Error = Error;
	fn try_from(val: String) -> Result<Id> {
		let id = val
			.parse()
			.map_err(|err| format!("id should be a number was '{val}'. Cause: {err}"))?;
		Ok(Id(id))
	}
}
