//! Defines the `aip.time` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.time` module exposes functions to retrieve current timestamps and dates.
//!
//! ### Functions
//!
//! - `aip.time.now_iso_utc(): string` - Current time as RFC3339 (UTC, second precision).
//!
//! - `aip.time.now_iso_local(): string` - Current time as RFC3339 (Local, second precision).
//!
//! - `aip.time.now_iso_utc_micro(): string` - Current time as RFC3339 (UTC, microsecond precision).
//!
//! - `aip.time.now_iso_local_micro(): string` - Current time as RFC3339 (Local, microsecond precision).
//!
//! - `aip.time.now_utc_micro(): integer` - Current epoch time in microseconds (UTC).
//!
//! - `aip.time.today_utc(): string` - Current date in UTC as `YYYY-MM-DD`.
//!
//! - `aip.time.today_local(): string` - Current date in Local time as `YYYY-MM-DD`.

use crate::Result;
use crate::runtime::Runtime;
use crate::support;
use mlua::{Lua, Table, Value};

/// Initializes the `time` Lua module.
///
/// Registers all time functions in the module table.
pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	table.set("now_iso_utc", lua.create_function(lua_now_iso_utc)?)?;
	table.set("now_iso_local", lua.create_function(lua_now_iso_local)?)?;
	table.set("now_iso_utc_micro", lua.create_function(lua_now_iso_utc_micro)?)?;
	table.set("now_iso_local_micro", lua.create_function(lua_now_iso_local_micro)?)?;

	table.set("now_utc_micro", lua.create_function(lua_now_utc_micro)?)?;

	table.set("today_utc", lua.create_function(lua_today_utc)?)?;
	table.set("today_local", lua.create_function(lua_today_local)?)?;

	Ok(table)
}

// region:    --- Lua Fns

fn lua_now_iso_utc(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let s = support::time::now_rfc3339_utc_sec().map_err(mlua::Error::external)?;
	let s = lua.create_string(&s)?;
	Ok(Value::String(s))
}

fn lua_now_iso_local(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let s = support::time::now_rfc3339_local_sec().map_err(mlua::Error::external)?;
	let s = lua.create_string(&s)?;
	Ok(Value::String(s))
}

fn lua_now_iso_utc_micro(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let s = support::time::now_rfc3339_utc_micro().map_err(mlua::Error::external)?;
	let s = lua.create_string(&s)?;
	Ok(Value::String(s))
}

fn lua_now_iso_local_micro(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let s = support::time::now_rfc3339_local_micro().map_err(mlua::Error::external)?;
	let s = lua.create_string(&s)?;
	Ok(Value::String(s))
}

fn lua_now_utc_micro(_lua: &Lua, _: ()) -> mlua::Result<Value> {
	let v = support::time::now_micro();
	Ok(Value::Integer(v))
}

fn lua_today_utc(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let s = support::time::today_utc().map_err(mlua::Error::external)?;
	let s = lua.create_string(&s)?;
	Ok(Value::String(s))
}

fn lua_today_local(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let s = support::time::today_local().map_err(mlua::Error::external)?;
	let s = lua.create_string(&s)?;
	Ok(Value::String(s))
}

// endregion: --- Lua Fns

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_time;
	use time::format_description;
	use time::format_description::well_known::Rfc3339;
	use time::{OffsetDateTime, UtcOffset};

	const LUA_MOD_NAME: &str = "time";

	#[tokio::test]
	async fn test_lua_time_now_iso_utc() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_time::init_module, LUA_MOD_NAME).await?;

		// -- Exec
		let res = eval_lua(&lua, r#"return aip.time.now_iso_utc()"#)?;

		// -- Check
		let s = res.as_str().ok_or("Should be string")?;
		let dt = OffsetDateTime::parse(s, &Rfc3339)?;
		assert_eq!(dt.offset(), UtcOffset::UTC);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_time_now_iso_local() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_time::init_module, LUA_MOD_NAME).await?;

		// -- Exec
		let res = eval_lua(&lua, r#"return aip.time.now_iso_local()"#)?;

		// -- Check
		let s = res.as_str().ok_or("Should be string")?;
		let _dt = OffsetDateTime::parse(s, &Rfc3339)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_time_now_iso_utc_micro() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_time::init_module, LUA_MOD_NAME).await?;

		// -- Exec
		let res = eval_lua(&lua, r#"return aip.time.now_iso_utc_micro()"#)?;

		// -- Check
		let s = res.as_str().ok_or("Should be string")?;
		let _dt = OffsetDateTime::parse(s, &Rfc3339)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_time_now_utc_micro() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_time::init_module, LUA_MOD_NAME).await?;

		// -- Exec
		let res = eval_lua(&lua, r#"return aip.time.now_utc_micro()"#)?;

		// -- Check
		let v = res.as_i64().ok_or("Should be integer")?;
		assert!(v > 0);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_time_today_utc_and_local() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_time::init_module, LUA_MOD_NAME).await?;
		let fmt = format_description::parse("[year]-[month]-[day]")?;

		// -- Exec
		let utc = eval_lua(&lua, r#"return aip.time.today_utc()"#)?;
		let local = eval_lua(&lua, r#"return aip.time.today_local()"#)?;

		// -- Check
		for s in [utc, local] {
			let s = s.as_str().ok_or("Should be string")?;
			assert_eq!(s.len(), 10);
			assert_eq!(s.chars().nth(4), Some('-'));
			assert_eq!(s.chars().nth(7), Some('-'));
			let _ = time::Date::parse(s, &fmt)?;
		}
		Ok(())
	}
}

// endregion: --- Tests
