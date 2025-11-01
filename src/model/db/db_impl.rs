use crate::model::Result;
use crate::model::db::rt_db_setup::recreate_db;
use modql::SqliteFromRow;
use rusqlite::types::FromSql;
use rusqlite::{Connection, OptionalExtension, Params};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Db {
	con: Arc<Mutex<Connection>>,
}

/// Constructor & Setup
impl Db {
	pub fn new() -> Result<Self> {
		// let con = Connection::open(".mock-db.sqlite")?;
		let con = Connection::open_in_memory()?;
		let con = Arc::new(Mutex::new(con));

		Ok(Self { con })
	}

	pub fn recreate(&self) -> Result<()> {
		let con = self.con.lock()?;
		recreate_db(&con)?;
		Ok(())
	}
}

// Executors
impl Db {
	/// Execute a parameterized sql with its params, and return the number of rows affected
	/// returns: number of rows affected
	pub fn exec(&self, sql: &str, params: impl Params) -> Result<usize> {
		let conn_g = self.con.lock()?;

		let row_affected = conn_g.execute(sql, params)?;
		Ok(row_affected)
	}

	/// Perform a sql exec and return the first row and first value as num
	/// NOTE: This is useful for query with RETURNING ID
	/// e.g., `db.exec_as_num("select count(*) from person", [] )`
	pub fn exec_returning_num(&self, sql: &str, params: impl Params) -> Result<i64> {
		let conn_g = self.con.lock()?;

		let mut stmt = conn_g.prepare(sql)?;
		// Note: Assume the first column is the id to be returned.
		let id = stmt.query_row(params, |r| r.get::<_, i64>(0))?;

		Ok(id)
	}

	/// Perform a sql exec and returns the first value of the first row and
	/// cast it to the type T
	/// ```
	/// let sql = r#"
	/// SELECT conv.cfile_id
	/// FROM conv
	/// JOIN space ON conv.space_id = space.id
	/// WHERE space.id = ?1
	/// ORDER BY conv.last_open DESC
	/// LIMIT 1;
	/// "#;
	/// let cfile_id: Option<Id> = mm.main_db().exec_as(sql, (space_id,))?;
	/// ```
	pub fn exec_returning_as<T: FromSql>(&self, sql: &str, params: impl Params) -> Result<T> {
		let conn_g = self.con.lock()?;

		let mut stmt = conn_g.prepare(sql)?;
		// Note: Assume the first column is the id to be returned.
		let res = stmt.query_row(params, |r| r.get::<_, T>(0))?;

		Ok(res)
	}

	pub fn exec_returning_as_optional<T: FromSql>(&self, sql: &str, params: impl Params) -> Result<Option<T>> {
		let conn_g = self.con.lock()?;

		let mut stmt = conn_g.prepare(sql)?;
		// Note: Assume the first column is the id to be returned.
		let res = stmt.query_row(params, |r| r.get::<_, T>(0)).optional()?;

		Ok(res)
	}

	/// Fetch the first row and cast to to Option<T>
	/// NOTE: This assume the sql would have the LIMIT 1 added
	/// TODO: Might want to add the LIMIT 1 if not already (not sure)
	pub fn fetch_first<P, T>(&self, sql: &str, params: P) -> Result<Option<T>>
	where
		P: Params,
		T: SqliteFromRow,
	{
		let all: Vec<T> = self.fetch_all(sql, params)?;

		Ok(all.into_iter().next())
	}

	pub fn fetch_all<P, T>(&self, sql: &str, params: P) -> Result<Vec<T>>
	where
		P: Params,
		T: SqliteFromRow,
	{
		let conn_g = self.con.lock()?;
		let mut stmt = conn_g.prepare(sql)?;
		let iter = stmt.query_and_then(params, |r| T::sqlite_from_row(r))?;
		let mut res = Vec::new();
		for item in iter {
			res.push(item?)
		}
		Ok(res)
	}
}
