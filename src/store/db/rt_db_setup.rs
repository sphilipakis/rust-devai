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
						
						-- Step Timestamps
						start       INTEGER,
						ba_start    INTEGER,	-- Before All start/end
						ba_end      INTEGER,
						tasks_start INTEGER,  -- All tasks start/end
						tasks_end   INTEGER,
						aa_start    INTEGER,  -- After All start/end
						aa_end      INTEGER,
						end         INTEGER,

						agent_name  TEXT,
						agent_path  TEXT,

						model       TEXT,
						concurrency INTEGER,

						total_cost  REAL,

						label TEXT

				) STRICT",
		(), // empty list of parameters.
	)?;
	con.execute(
		"
		CREATE INDEX IF NOT EXISTS idx_run_uid ON run(uid);
		",
		(),
	)?;

	// -- Create the Task table
	con.execute(
		"CREATE TABLE IF NOT EXISTS task (
						id     INTEGER PRIMARY KEY,
						uid    BLOB NOT NULL,

						ctime  INTEGER NOT NULL,
						mtime  INTEGER NOT NULL,								

						run_id INTEGER NOT NULL,
						idx    INTEGER, -- Relative to the run (as created by Run)
						
						-- Step Timestamps
						start         INTEGER,
						data_start    INTEGER,    
						data_end      INTEGER,
						ai_start      INTEGER,
						ai_end        INTEGER,
						output_start  INTEGER,
						output_end    INTEGER,
						end           INTEGER,

						model   TEXT,

						-- Usage Raw
						usage BLOB, -- jsonb, to have raw usage
						-- Usage Values
						tk_prompt_total             INTEGER,
						tk_prompt_cached            INTEGER,
						tk_prompt_cache_creation    INTEGER,
						tk_completion_total         INTEGER,
						tk_completion_reasoning     INTEGER,
						
						cost  REAL,

						label  TEXT

				) STRICT",
		(), // empty list of parameters.
	)?;
	con.execute(
		"
		CREATE INDEX IF NOT EXISTS idx_task_uid ON task(uid);
		",
		(),
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

						kind    TEXT,  -- UserPrint, SysInfo, SysWarn, SysDebug

						stage   TEXT, 

						step    TEXT, 
						
						message TEXT

				) STRICT",
		(), // empty list of parameters.
	)?;
	con.execute(
		"
		CREATE INDEX IF NOT EXISTS idx_log_uid ON log(uid);
		",
		(),
	)?;

	Ok(())
}

// endregion: --- Support
