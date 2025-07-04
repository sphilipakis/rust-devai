use crate::store::Result;
use rusqlite::Connection;

pub fn recreate_db(con: &Connection) -> Result<()> {
	create_schema(con)?;
	Ok(())
}

// region:    --- Support

fn create_schema(con: &Connection) -> Result<()> {
	// -- Create the Run table
	con.execute(
		"CREATE TABLE IF NOT EXISTS run (
						id     INTEGER PRIMARY KEY,
						uid    BLOB NOT NULL,

						ctime  INTEGER NOT NULL,
						mtime  INTEGER NOT NULL,						
						
						start  INTEGER,
						end    INTEGER,

						-- Before All start/end
						ba_start INTEGER,
						ba_end   INTEGER,

						-- All tasks start/end
						tasks_start INTEGER,
						tasks_end   INTEGER,

						-- After All start/end
						aa_start INTEGER,
						aa_end   INTEGER,

						agent_name TEXT,
						agent_path TEXT,
						label TEXT

				) STRICT",
		(), // empty list of parameters.
	)?;

	// -- Create the Task table
	con.execute(
		"CREATE TABLE IF NOT EXISTS task (
						id     INTEGER PRIMARY KEY,
						uid    BLOB NOT NULL,

						ctime  INTEGER NOT NULL,
						mtime  INTEGER NOT NULL,								

						run_id INTEGER NOT NULL,
						num    INTEGER, -- start at 1
						
						-- Full Run start/end
						start  INTEGER,
						end    INTEGER,

						-- Before All start/end
						ba_start INTEGER,
						ba_end   INTEGER,
						
						-- All tasks start/end
						-- So, first start, and last end
						tasks_start INTERGER,
						tasks_end   INTEGER,

						-- After All start/end
						aa_start INTERGER,
						aa_end   INTEGER,						

						
						label  TEXT

				) STRICT",
		(), // empty list of parameters.
	)?;

	// -- Create the Log message
	con.execute(
		"CREATE TABLE IF NOT EXISTS log (
						id      INTEGER PRIMARY KEY,
						uid     BLOB NOT NULL,

						ctime   INTEGER NOT NULL,
						mtime   INTEGER NOT NULL,								

						run_id  INTEGER NOT NULL, -- Should always belong to a run
						task_id INTEGER,          -- Might belong to a task

						stage   TEXT, 
						
						message  TEXT

				) STRICT",
		(), // empty list of parameters.
	)?;

	Ok(())
}

// endregion: --- Support
