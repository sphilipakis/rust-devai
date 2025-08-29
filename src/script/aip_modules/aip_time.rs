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
//! ```lua
//! aip.time.now_iso_utc(): string            -- RFC 3339 UTC (seconds precision)
//! -- e.g., "2025-08-23T14:35:12Z"
//!
//! aip.time.now_iso_local(): string          -- RFC 3339 local time (seconds precision)
//! -- e.g., "2025-08-23T09:35:12-05:00"
//!
//! aip.time.now_iso_utc_micro(): string      -- RFC 3339 UTC (microseconds)
//! -- e.g., "2025-08-23T14:35:12.123456Z"
//!
//! aip.time.now_iso_local_micro(): string    -- RFC 3339 local time (microseconds)
//! -- e.g., "2025-08-23T09:35:12.123456-05:00"
//!
//! aip.time.now_utc_micro(): integer         -- epoch microseconds (UTC)
//! -- e.g., 1766561712123456
//!
//! aip.time.today_utc(): string              -- weekday + date (UTC)
//! -- e.g., "Saturday 2025-08-23"
//!
//! aip.time.today_local(): string            -- weekday + date (local)
//! -- e.g., "Saturday 2025-08-23"
//!
//! aip.time.today_iso_utc(): string          -- "YYYY-MM-DD" (UTC)
//! -- e.g., "2025-08-23"
//!
//! aip.time.today_iso_local(): string        -- "YYYY-MM-DD" (local)
//! -- e.g., "2025-08-23"
//!
//! aip.time.weekday_utc(): string            -- weekday name (UTC)
//! -- e.g., "Saturday"
//!
//! aip.time.weekday_local(): string          -- weekday name (local)
//! -- e.g., "Saturday"
//!
//! aip.time.local_tz_id(): string            -- IANA timezone id for local zone
//! -- e.g., "America/Los_Angeles"
//! ```
use crate::Result;
use crate::runtime::Runtime;
use crate::support;
use mlua::{Lua, Table, Value};
use time::{OffsetDateTime, UtcOffset};
use time_tz::TimeZone as _;
use time_tz::system::get_timezone;

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
	table.set("today_iso_utc", lua.create_function(lua_today_iso_utc)?)?;
	table.set("today_iso_local", lua.create_function(lua_today_iso_local)?)?;
	table.set("weekday_utc", lua.create_function(lua_weekday_utc)?)?;
	table.set("weekday_local", lua.create_function(lua_weekday_local)?)?;
	table.set("local_tz_id", lua.create_function(lua_local_tz_id)?)?;

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
	let now_utc = OffsetDateTime::now_utc();
	let date = now_utc.date();
	let weekday = format!("{:?}", date.weekday());
	let iso = format!("{:04}-{:02}-{:02}", date.year(), date.month() as u8, date.day());
	let s = lua.create_string(format!("{weekday} {iso}"))?;
	Ok(Value::String(s))
}

fn lua_today_local(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let now_utc = OffsetDateTime::now_utc();
	let local_offset = UtcOffset::current_local_offset().map_err(mlua::Error::external)?;
	let date = now_utc.to_offset(local_offset).date();
	let weekday = format!("{:?}", date.weekday());
	let iso = format!("{:04}-{:02}-{:02}", date.year(), date.month() as u8, date.day());
	let s = lua.create_string(format!("{weekday} {iso}"))?;
	Ok(Value::String(s))
}

fn lua_today_iso_utc(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let s = support::time::today_utc().map_err(mlua::Error::external)?;
	let s = lua.create_string(&s)?;
	Ok(Value::String(s))
}

fn lua_today_iso_local(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let s = support::time::today_local().map_err(mlua::Error::external)?;
	let s = lua.create_string(&s)?;
	Ok(Value::String(s))
}

fn lua_weekday_utc(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let now_utc = OffsetDateTime::now_utc();
	let weekday = format!("{:?}", now_utc.date().weekday());
	let s = lua.create_string(&weekday)?;
	Ok(Value::String(s))
}

fn lua_weekday_local(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let now_utc = OffsetDateTime::now_utc();
	let local_offset = UtcOffset::current_local_offset().map_err(mlua::Error::external)?;
	let weekday = format!("{:?}", now_utc.to_offset(local_offset).date().weekday());
	let s = lua.create_string(&weekday)?;
	Ok(Value::String(s))
}

fn lua_local_tz_id(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let tz_id = get_timezone().map(|tz| tz.name()).unwrap_or("UTC");
	let s = lua.create_string(tz_id)?;
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
	use time::{Date, OffsetDateTime, UtcOffset};

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
		let valid_weekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

		// -- Exec
		let utc = eval_lua(&lua, r#"return aip.time.today_utc()"#)?;
		let local = eval_lua(&lua, r#"return aip.time.today_local()"#)?;

		// -- Check
		for s in [utc, local] {
			let s = s.as_str().ok_or("Should be string")?;
			let parts: Vec<&str> = s.splitn(2, ' ').collect();
			assert_eq!(parts.len(), 2, "Should contain 'Weekday YYYY-MM-DD'");
			let weekday = parts[0];
			let date_s = parts[1];

			assert!(valid_weekdays.contains(&weekday), "Invalid weekday: {weekday}");
			assert_eq!(date_s.len(), 10);
			assert_eq!(date_s.chars().nth(4), Some('-'));
			assert_eq!(date_s.chars().nth(7), Some('-'));

			let date = Date::parse(date_s, &fmt)?;
			let expected = format!("{:?}", date.weekday());
			assert_eq!(weekday, expected, "Weekday should match the date's weekday");
		}
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_time_today_iso_utc_and_local() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_time::init_module, LUA_MOD_NAME).await?;
		let fmt = format_description::parse("[year]-[month]-[day]")?;

		// -- Exec
		let utc = eval_lua(&lua, r#"return aip.time.today_iso_utc()"#)?;
		let local = eval_lua(&lua, r#"return aip.time.today_iso_local()"#)?;

		// -- Check
		for s in [utc, local] {
			let s = s.as_str().ok_or("Should be string")?;
			assert_eq!(s.len(), 10);
			assert_eq!(s.chars().nth(4), Some('-'));
			assert_eq!(s.chars().nth(7), Some('-'));
			let _ = Date::parse(s, &fmt)?;
		}
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_time_weekday_utc_and_local() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_time::init_module, LUA_MOD_NAME).await?;
		let valid_weekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

		// -- Exec
		let utc = eval_lua(&lua, r#"return aip.time.weekday_utc()"#)?;
		let local = eval_lua(&lua, r#"return aip.time.weekday_local()"#)?;

		// -- Check
		for s in [utc, local] {
			let s = s.as_str().ok_or("Should be string")?;
			assert!(valid_weekdays.contains(&s), "Invalid weekday: {s}");
		}
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_time_local_tz_id() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_time::init_module, LUA_MOD_NAME).await?;

		// -- Exec
		let res = eval_lua(&lua, r#"return aip.time.local_tz_id()"#)?;

		// -- Check
		let s = res.as_str().ok_or("Should be string")?;
		assert!(!s.is_empty());
		Ok(())
	}
}

// endregion: --- Tests
