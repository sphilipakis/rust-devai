// NOTES
//   - Make sure to use the `now_utc_fmt` for Rfc3339 for time, otherwise, rusqlite format it wrong.

use super::DbBmc;
use crate::support::time::now_unix_time_us;
use modql::field::{SqliteField, SqliteFields};
use uuid::Uuid;

/// This method must be called when a model controller intends to create its entity.
pub fn prep_fields_for_create<MC>(fields: &mut SqliteFields)
where
	MC: DbBmc,
{
	add_timestamps_for_create(fields);
	let uid = Uuid::now_v7();
	fields.push(SqliteField::new("uid", uid.into()));
}

/// This method must be calledwhen a Model Controller plans to update its entity.
pub fn prep_fields_for_update<MC>(fields: &mut SqliteFields)
where
	MC: DbBmc,
{
	add_timestamps_for_update(fields);
}

/// Update the timestamps info for create
/// (e.g., cid, ctime, and mid, mtime will be updated with the same values)
fn add_timestamps_for_create(fields: &mut SqliteFields) {
	let now = now_unix_time_us();
	fields.push(SqliteField::new("ctime", now.into()));
	fields.push(SqliteField::new("mtime", now.into()));
}

/// Update the timestamps info only for update.
/// (.e.g., only mid, mtime will be udpated)
fn add_timestamps_for_update(fields: &mut SqliteFields) {
	let now = now_unix_time_us();
	fields.push(SqliteField::new("mtime", now.into()));
}
