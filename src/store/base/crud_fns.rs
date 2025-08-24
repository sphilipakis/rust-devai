use crate::hub::get_hub;
use crate::store::Id;
use crate::store::ModelManager;
use crate::store::Result;
use crate::store::base::DbBmc;
use crate::store::base::prep_fields::prep_fields_for_create;
use crate::store::base::prep_fields::prep_fields_for_create_uid_included;
use crate::store::base::prep_fields::prep_fields_for_update;
use crate::support::consts;
use modql::SqliteFromRow;
use modql::field::HasSqliteFields;
use modql::field::SqliteFields;
use modql::filter::ListOptions;
use rusqlite::ToSql;
use uuid::Uuid;

pub fn create<MC>(mm: &ModelManager, fields: SqliteFields) -> Result<Id>
where
	MC: DbBmc,
{
	create_inner::<MC>(mm, fields, true)
}

pub fn create_uid_included<MC>(mm: &ModelManager, fields: SqliteFields) -> Result<Id>
where
	MC: DbBmc,
{
	create_inner::<MC>(mm, fields, false)
}

pub fn create_where_not_exists<MC>(
	mm: &ModelManager,
	mut fields: SqliteFields,
	not_exists_fields: SqliteFields,
	not_exists_extra_static: Option<&str>, // hack for now for the task_id IS NULL
) -> Result<Option<Id>>
where
	MC: DbBmc,
{
	prep_fields_for_create::<MC>(&mut fields);

	let table = MC::table_ref();
	let columns = fields.sql_columns();
	let placeholders = fields.sql_placeholders();
	let mut where_clause = not_exists_fields
		.fields()
		.iter()
		.map(|f| format!("\"{}\" = ?", f.iden)) // won't work with rel.col
		.collect::<Vec<_>>()
		.join(" AND ");

	if let Some(extra_static) = not_exists_extra_static {
		where_clause.push_str(&format!(" AND {extra_static}"));
	}

	let sql = format!(
		"
INSERT INTO {table} ({columns}) SELECT {placeholders}
WHERE NOT EXISTS (
		SELECT 1 FROM {table} where {where_clause}
)
RETURNING id",
	);

	// -- Execute the command
	let fields = fields.extended(not_exists_fields);
	let values = fields.values_as_dyn_to_sql_vec();
	let db = mm.db();

	let id: Option<i64> = db.exec_returning_as_optional(&sql, &*values)?;

	Ok(id.map(|id| id.into()))
}

fn create_inner<MC>(mm: &ModelManager, mut fields: SqliteFields, generate_uuid: bool) -> Result<Id>
where
	MC: DbBmc,
{
	if generate_uuid {
		prep_fields_for_create::<MC>(&mut fields);
	} else {
		prep_fields_for_create_uid_included(&mut fields);
	}

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

	// -- Publish Change Event
	get_hub().publish_rt_model_change_sync();

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

	// -- Publish Change Event
	get_hub().publish_rt_model_change_sync();

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
		E::sqlite_columns_for_select(),
		MC::table_ref(),
	);

	// -- Exec query
	let db = mm.db();
	let entity: E = db
		.fetch_first(&sql, [(&id)])?
		.ok_or_else(|| format!("Cannot get entity '{}'", MC::TABLE))?;

	Ok(entity)
}

pub fn get_by_uid<MC, E>(mm: &ModelManager, uid: Uuid) -> Result<E>
where
	MC: DbBmc,
	E: SqliteFromRow + Unpin + Send,
	E: HasSqliteFields,
{
	// -- Select
	let sql = format!(
		"SELECT {} FROM {} WHERE uid = ? LIMIT 1",
		//
		E::sqlite_columns_for_select(),
		MC::table_ref(),
	);

	// -- Exec query
	let db = mm.db();
	let entity: E = db.fetch_first(&sql, [(&uid)])?.ok_or("Cannot get entity")?;

	Ok(entity)
}

pub fn get_uid<MC>(mm: &ModelManager, id: Id) -> Result<Uuid>
where
	MC: DbBmc,
{
	let sql = format!("SELECT uid FROM {} WHERE id = ? LIMIT 1", MC::table_ref());

	// -- Exec query
	let db = mm.db();
	let uid: Uuid = db.exec_returning_as(&sql, (id,))?;

	Ok(uid)
}

pub fn get_id_for_uid<MC>(mm: &ModelManager, uid: Uuid) -> Result<Id>
where
	MC: DbBmc,
{
	let sql = format!("SELECT id FROM {} WHERE uid = ? LIMIT 1", MC::table_ref());

	// -- Exec query
	let db = mm.db();
	let id: Id = db.exec_returning_as(&sql, (uid,))?;

	Ok(id)
}

#[allow(unused)]
pub fn first<MC, E>(
	mm: &ModelManager,
	list_options: Option<ListOptions>,
	filter_fields: Option<SqliteFields>,
) -> Result<Option<E>>
where
	MC: DbBmc,
	E: SqliteFromRow + Unpin + Send,
	E: HasSqliteFields,
{
	let list_options = if let Some(list_options) = list_options {
		list_options.with_limit(1)
	} else {
		ListOptions::from_limit(1)
	};
	let entities = list::<MC, E>(mm, Some(list_options), filter_fields)?;
	Ok(entities.into_iter().next())
}

pub fn list<MC, E>(
	mm: &ModelManager,
	list_options: Option<ListOptions>,
	filter_fields: Option<SqliteFields>,
) -> Result<Vec<E>>
where
	MC: DbBmc,
	E: SqliteFromRow + Unpin + Send,
	E: HasSqliteFields,
{
	let list_options = list_options.unwrap_or_default();
	let limit = list_options.limit.unwrap_or(consts::DEFAULT_LIST_LIMIT);
	let order_by = list_options
		.order_bys
		.map(|ob| ob.join_for_sql())
		.unwrap_or_else(|| "id".to_string());
	// TODO: add the offset

	// -- Select
	let (sql, params) = if let Some(filter_fields) = filter_fields.as_ref() {
		// NOTE: For now only support =
		let where_clause = filter_fields
			.fields()
			.iter()
			.map(|f| format!("\"{}\" = ?", f.iden)) // won't work with rel.col
			.collect::<Vec<_>>()
			.join(" AND ");

		let sql = format!(
			"SELECT {} FROM {} WHERE {} ORDER BY {order_by} LIMIT {limit} ",
			E::sql_columns(),
			MC::table_ref(),
			where_clause,
		);

		(sql, filter_fields.values_as_dyn_to_sql_vec())
	} else {
		let sql = format!(
			"SELECT {} FROM {} ORDER BY {order_by} LIMIT {limit} ",
			E::sql_columns(),
			MC::table_ref()
		);
		(sql, Vec::new())
	};

	// -- Exec query
	let db = mm.db();
	let entities: Vec<E> = db.fetch_all(&sql, &*params)?;

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

// 	// -- Execute the command
// 	let values = fields.values_as_dyn_to_sql_vec();
// 	let db = mm.db();

// 	let id = db.exec_returning_num(&sql, &*values)?;

// 	Ok(123.into())
// }
