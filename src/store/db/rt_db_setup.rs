use crate::store::Result;
use rusqlite::Connection;

// Some notes:
// - Right now, memory db only, might become persistent at the session level
// - All table have `id` which is use for same db joins, and `uid` which is a uuid blob,
//   when need to shared out of rust or accross db
// - `id` is not with `AUTOINCREMENT` for making ids are not reused if row get deleted

pub fn recreate_db(con: &Connection) -> Result<()> {
	create_schema(con)?;
	Ok(())
}

// region:    --- Table SQLs

const RUN_TABLE: &str = "
CREATE TABLE IF NOT EXISTS run (
		id     INTEGER PRIMARY KEY AUTOINCREMENT,
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

) STRICT";

const TASK_TABLE: &str = "
CREATE TABLE IF NOT EXISTS task (
		id     INTEGER PRIMARY KEY AUTOINCREMENT,
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

		model_ov   TEXT,

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

) STRICT";

const LOG_TABLE: &str = "
CREATE TABLE IF NOT EXISTS log (
		id      INTEGER PRIMARY KEY AUTOINCREMENT,
		uid     BLOB NOT NULL,

		ctime   INTEGER NOT NULL,
		mtime   INTEGER NOT NULL,							

		run_id  INTEGER NOT NULL, -- Should always belong to a run
		task_id INTEGER,          -- Might belong to a task

		kind    TEXT,  -- UserPrint, SysInfo, SysWarn, SysDebug

		stage   TEXT, 

		step    TEXT, 
		
		message TEXT
) STRICT";

const ALL_MAIN_TABLES: &[(&str, &str)] = &[
	//
	("run", RUN_TABLE),
	("task", TASK_TABLE),
	("log", LOG_TABLE),
];

// endregion: --- Table SQLs

// region:    --- Support

fn create_schema(con: &Connection) -> Result<()> {
	for (name, table_sql) in ALL_MAIN_TABLES {
		con.execute(table_sql, ())?;
		con.execute(
			&format!(
				"
		CREATE INDEX IF NOT EXISTS idx_{name}_uid ON {name}(uid);
		"
			),
			(),
		)?;
	}

	Ok(())
}

// endregion: --- Support
