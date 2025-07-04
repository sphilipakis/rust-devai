use crate::store::Id;
use crate::store::ModelManager;
use crate::store::Result;
use crate::store::base::DbBmc;
use crate::store::base::prep_fields::prep_fields_for_create;
use crate::store::base::prep_fields::prep_fields_for_update;
use modql::SqliteFromRow;
use modql::field::HasSqliteFields;
use modql::field::SqliteFields;
use modql::filter::ListOptions;
use rusqlite::ToSql;

pub fn create<MC>(mm: &ModelManager, mut fields: SqliteFields) -> Result<Id>
where
	MC: DbBmc,
{
	prep_fields_for_create::<MC>(&mut fields);

	// -- Build sql
	let sql = format!(
		"INSERT INTO {} ({}) VALUES ({}) RETURNING id",
		MC::table_ref(),
		fields.sql_columns(),
		fields.sql_placeholders()
	);

	// -- Execute the command
	let values = fields.values_as_dyn_to_sql_vec();
	let db = mm.db();

	let id = db.exec_returning_num(&sql, &*values)?;

	Ok(id.into())
}

pub fn update<MC>(mm: &ModelManager, id: Id, mut fields: SqliteFields) -> Result<usize>
where
	MC: DbBmc,
{
	prep_fields_for_update::<MC>(&mut fields);

	// -- Build sql
	let sql = format!("UPDATE {} SET {} WHERE id = ?", MC::table_ref(), fields.sql_setters(),);

	// -- Execute the command
	let mut values = fields.values_as_dyn_to_sql_vec();
	values.push((&id) as &dyn ToSql);
	let db = mm.db();

	let count = db.exec(&sql, &*values)?;

	Ok(count)
}

pub fn get<MC, E>(mm: &ModelManager, id: Id) -> Result<E>
where
	MC: DbBmc,
	E: SqliteFromRow + Unpin + Send,
	E: HasSqliteFields,
{
	// -- Select
	let sql = format!(
		"SELECT {} FROM {} WHERE id = ? LIMIT 1",
		//
		E::sql_columns(),
		MC::table_ref(),
	);

	// -- Exec query
	let db = mm.db();
	let entity: E = db.fetch_first(&sql, [(&id)])?.ok_or("Cannot get entity")?;

	Ok(entity)
}

pub fn list<MC, E>(mm: &ModelManager, list_options: Option<ListOptions>) -> Result<Vec<E>>
where
	MC: DbBmc,
	E: SqliteFromRow + Unpin + Send,
	E: HasSqliteFields,
{
	let list_options = list_options.unwrap_or_default();
	let limit = list_options.limit.unwrap_or(300);
	let order_by = list_options
		.order_bys
		.map(|ob| ob.join_for_sql())
		.unwrap_or_else(|| "id".to_string());
	// TODO: add the offset

	// -- Select
	let sql = format!(
		"SELECT {} FROM {} ORDER BY {order_by} LIMIT {limit} ",
		E::sql_columns(),
		MC::table_ref()
	);

	// -- Exec query
	let db = mm.db();
	let entities: Vec<E> = db.fetch_all(&sql, [])?;

	Ok(entities)
}

// pub fn list<MC>(mm: &ModelManager) -> Result<Id>
// where
// 	MC: DbBmc,
// {
// 	// -- Build sql
// 	let sql = format!(
// 		"INSERT INTO {} ({}) VALUES ({}) RETURNING id",
// 		MC::table_ref(),
// 		fields.sql_columns(),
// 		fields.sql_placeholders()
// 	);
// 	println!("->> SQL: {sql}");

// 	// -- Execute the command
// 	let values = fields.values_as_dyn_to_sql_vec();
// 	let db = mm.db();

// 	let id = db.exec_returning_num(&sql, &*values)?;

// 	Ok(123.into())
// }
