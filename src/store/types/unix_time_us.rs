use crate::derive_simple_data_type;
use crate::{Error, Result};

derive_simple_data_type! {
	pub struct UnixTimeUs(i64);
}
impl UnixTimeUs {
	pub fn as_i64(&self) -> i64 {
		self.0
	}
}

// from &i64
impl From<&i64> for UnixTimeUs {
	fn from(val: &i64) -> UnixTimeUs {
		UnixTimeUs(*val)
	}
}

impl TryFrom<String> for UnixTimeUs {
	type Error = Error;
	fn try_from(val: String) -> Result<UnixTimeUs> {
		let id = val
			.parse()
			.map_err(|err| format!("id should be a number was '{val}'. Cause: {err}"))?;
		Ok(UnixTimeUs(id))
	}
}
