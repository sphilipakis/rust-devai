use crate::store::Result;
use rusqlite::Connection;

// Some notes:
// - Currently, the database is in-memory only, but it may become persistent at the session level.
// - All tables have an `id` used for same-db joins, and a `uid` which is a UUID blob,
//   intended for sharing outside of Rust or across databases.
// - `id` uses `AUTOINCREMENT` to ensure IDs are not reused if a row is deleted.
// - `MAIN_TABLES` are the main database tables for all metadata. They are designed to be relatively small and to scale well.
// - `CONTENT_TABLES` are designed to hold larger content and may have different trimming/cleaning strategies.
//    - A future strategy could involve having a set of content tables per run, using the b58 run.uid suffix. This would make it very fast to clean up old ones.
// - References between these two sets of tables are by `uid`, as they may eventually reside in different databases.

pub fn recreate_db(con: &Connection) -> Result<()> {
	create_schema(con)?;
	Ok(())
}

// region:    --- Main Tables

const RUN_TABLE: (&str, &str) = (
	"run",
	"
CREATE TABLE IF NOT EXISTS run (
		id          INTEGER PRIMARY KEY AUTOINCREMENT,
		uid         BLOB NOT NULL,

		parent_id   INTEGER,

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

		-- End state & Data
		end_state        TEXT,
		end_err_id       INTEGER,
		end_skip_reason  TEXT,

		agent_name  TEXT,
		agent_path  TEXT,

		model       TEXT,
		concurrency INTEGER,

		total_cost  REAL,

		label TEXT

) STRICT",
);

const TASK_TABLE: (&str, &str) = (
	"task",
	"
CREATE TABLE IF NOT EXISTS task (
		id     INTEGER PRIMARY KEY AUTOINCREMENT,
		uid    BLOB NOT NULL,

		ctime  INTEGER NOT NULL,
		mtime  INTEGER NOT NULL,								

		run_id INTEGER NOT NULL,
		idx    INTEGER, -- Relative to the run (as created by Run)
		
		-- Step Timestamps
		start          INTEGER,
		data_start     INTEGER,    
		data_end       INTEGER,
		ai_start       INTEGER,
		ai_gen_start   INTEGER,
		ai_gen_end     INTEGER,
		ai_end         INTEGER,
		output_start   INTEGER,
		output_end     INTEGER,
		end            INTEGER,

		-- End state & Data
		end_state        TEXT,
		end_err_id       INTEGER,
		end_skip_reason  TEXT,

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

		label               TEXT,

		input_uid           BLOB,
		input_short         TEXT,
		input_has_display   INTEGER,

		output_uid          BLOB,
		output_short        TEXT,
		output_has_display  INTEGER

) STRICT",
);

const LOG_TABLE: (&str, &str) = (
	"log",
	"
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
) STRICT",
);

const ERR_TABLE: (&str, &str) = (
	"err",
	"
CREATE TABLE IF NOT EXISTS err (
		id       INTEGER PRIMARY KEY AUTOINCREMENT,
		uid      BLOB NOT NULL,

		ctime    INTEGER NOT NULL,
		mtime    INTEGER NOT NULL,

		run_id  INTEGER, -- for now, allow null, for global errors
		stage   TEXT,
		task_id INTEGER,

		typ      TEXT, -- 'text' | 'json'
		content  TEXT
) STRICT",
);

const ALL_MAIN_TABLES: &[(&str, &str)] = &[RUN_TABLE, TASK_TABLE, ERR_TABLE, LOG_TABLE];

// endregion: --- Main Tables

// region:    --- Content Tables

// NOTE: Currently, the idea is that content tables are for "larger content" and should be efficient to delete per run.
//       However, this concept is not fully realized yet. One idea is to have table names with a run_uid_b58 suffix,
//       allowing for very fast deletion. Implementing this will require additional code.
//       Also, at present, we do not have run_id or run_uid in these tables, which is problematic when we want to trim by run.
//       At least the main tables can do that by run.

const INOUT_TABLE: (&str, &str) = (
	"inout",
	"
CREATE TABLE IF NOT EXISTS inout (
		id       INTEGER PRIMARY KEY AUTOINCREMENT,
		uid      BLOB NOT NULL,

		ctime    INTEGER NOT NULL,
		mtime    INTEGER NOT NULL,							

		task_uid BLOB NOT NULL, -- might not be needed but used to guarantee on task

		kind     TEXT, -- In, Out

		typ      TEXT, -- 'text' | 'json'
		content  TEXT,
		display  TEXT
) STRICT",
);

const ALL_CONTENT_TABLES: &[(&str, &str)] = &[INOUT_TABLE];

// endregion: --- Content Tables

// region:    --- Support

fn create_schema(con: &Connection) -> Result<()> {
	for tables in [ALL_MAIN_TABLES, ALL_CONTENT_TABLES] {
		for (name, table_sql) in tables {
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
	}

	Ok(())
}

// endregion: --- Support
