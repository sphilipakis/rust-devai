use crate::store::Db;
use crate::store::Result;
use rusqlite::Connection;

pub fn recreate_db(con: &Connection) -> Result<()> {
	create_schema(con)?;
	Ok(())
}

// region:    --- Support

fn create_schema(con: &Connection) -> Result<()> {
	con.execute(
		"CREATE TABLE IF NOT EXISTS run (
						id     INTEGER PRIMARY KEY,
						uid    BLOB NOT NULL,

						ctime  INTEGER NOT NULL,
						mtime  INTEGER NOT NULL,						
						
						start  INTEGER,
						end    INTEGER,

						label  TEXT

				) STRICT",
		(), // empty list of parameters.
	)?;

	con.execute(
		"CREATE TABLE IF NOT EXISTS task (
						id     INTEGER PRIMARY KEY,
						uid    BLOB NOT NULL,

						ctime  INTEGER NOT NULL,
						mtime  INTEGER NOT NULL,								

						run_id INTEGER NOT NULL,
						num    INTEGER, -- start at 1
						
						start  INTEGER,
						end    INTEGER,

						
						label  TEXT

				) STRICT",
		(), // empty list of parameters.
	)?;

	Ok(())
}

// endregion: --- Support
