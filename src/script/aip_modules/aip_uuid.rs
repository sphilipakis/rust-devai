//! Defines the `aip.uuid` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.uuid` module exposes functions for generating various UUIDs and converting timestamped UUIDs.
//!
//! ### Functions
//!
//! - `aip.uuid.new(): string` - Generates a new UUID version 4. Alias for `new_v4`.
//! - `aip.uuid.new_v4(): string` - Generates a new UUID version 4.
//! - `aip.uuid.new_v7(): string` - Generates a new UUID version 7.
//! - `aip.uuid.new_v4_b64(): string` - Generates a new UUID version 4, standard Base64 encoded.
//! - `aip.uuid.new_v4_b64u(): string` - Generates a new UUID version 4, URL-safe Base64 encoded (no padding).
//! - `aip.uuid.new_v4_b58(): string` - Generates a new UUID version 4, Base58 encoded.
//! - `aip.uuid.new_v7_b64(): string` - Generates a new UUID version 7, standard Base64 encoded.
//! - `aip.uuid.new_v7_b64u(): string` - Generates a new UUID version 7, URL-safe Base64 encoded (no padding).
//! - `aip.uuid.new_v7_b58(): string` - Generates a new UUID version 7, Base58 encoded.
//! - `aip.uuid.to_time_epoch_ms(value: string | nil): integer | nil` - Converts a timestamped UUID string (V1, V6, V7) to milliseconds since Unix epoch. Returns `nil` if input is `nil`, not a valid UUID, or a UUID type without an extractable timestamp (e.g., V4).

use crate::runtime::Runtime;
use crate::script::support::into_option_string;
use mlua::{Lua, Table, Value};
use uuid::Uuid;

// region:    --- Lua Interface

/// Initializes the `uuid` Lua module.
///
/// Registers the UUID generation functions in the module table.
pub fn init_module(lua: &Lua, _runtime: &Runtime) -> crate::Result<Table> {
	let table = lua.create_table()?;

	table.set("new", lua.create_function(lua_new_v4)?)?;
	table.set("new_v4", lua.create_function(lua_new_v4)?)?;
	table.set("new_v7", lua.create_function(lua_new_v7)?)?;
	table.set("new_v4_b64", lua.create_function(lua_new_v4_b64)?)?;
	table.set("new_v4_b64u", lua.create_function(lua_new_v4_b64url_nopad)?)?;
	table.set("new_v4_b58", lua.create_function(lua_new_v4_b58)?)?;
	table.set("new_v7_b64", lua.create_function(lua_new_v7_b64)?)?;
	table.set("new_v7_b64u", lua.create_function(lua_new_v7_b64url_nopad)?)?;
	table.set("new_v7_b58", lua.create_function(lua_new_v7_b58)?)?;
	table.set("to_time_epoch_ms", lua.create_function(lua_to_time_epoch_ms)?)?;

	Ok(table)
}

// region:    --- Lua Functions

/// ## Lua Documentation aip.uuid.new
///
/// Generates a new UUID version 4. This is an alias for `aip.uuid.new_v4()`.
///
/// ```lua
/// -- API Signature
/// aip.uuid.new(): string
/// ```
///
/// ### Returns
///
/// `string`: The generated UUIDv4 as a string (e.g., "f47ac10b-58cc-4372-a567-0e02b2c3d479").
///
/// ### Example
///
/// ```lua
/// local id = aip.uuid.new()
/// print(id)
/// ```
///
/// ## Lua Documentation aip.uuid.new_v4
///
/// Generates a new UUID version 4.
///
/// ```lua
/// -- API Signature
/// aip.uuid.new_v4(): string
/// ```
///
/// ### Returns
///
/// `string`: The generated UUIDv4 as a string (e.g., "f47ac10b-58cc-4372-a567-0e02b2c3d479").
///
/// ### Example
///
/// ```lua
/// local id_v4 = aip.uuid.new_v4()
/// print(id_v4)
/// ```
fn lua_new_v4(_lua: &Lua, (): ()) -> mlua::Result<String> {
	Ok(uuid_extra::new_v4().to_string())
}

/// ## Lua Documentation aip.uuid.new_v7
///
/// Generates a new UUID version 7 (time-ordered).
///
/// ```lua
/// -- API Signature
/// aip.uuid.new_v7(): string
/// ```
///
/// ### Returns
///
/// `string`: The generated UUIDv7 as a string.
///
/// ### Example
///
/// ```lua
/// local id_v7 = aip.uuid.new_v7()
/// print(id_v7)
/// ```
fn lua_new_v7(_lua: &Lua, (): ()) -> mlua::Result<String> {
	Ok(uuid_extra::new_v7().to_string())
}

/// ## Lua Documentation aip.uuid.new_v4_b64
///
/// Generates a new UUID version 4 and encodes it using standard Base64.
///
/// ```lua
/// -- API Signature
/// aip.uuid.new_v4_b64(): string
/// ```
///
/// ### Returns
///
/// `string`: The Base64 encoded UUIDv4 string.
///
/// ### Example
///
/// ```lua
/// local id_v4_b64 = aip.uuid.new_v4_b64()
/// print(id_v4_b64)
/// ```
fn lua_new_v4_b64(_lua: &Lua, (): ()) -> mlua::Result<String> {
	Ok(uuid_extra::new_v4_b64())
}

/// ## Lua Documentation aip.uuid.new_v4_b64u
///
/// Generates a new UUID version 4 and encodes it using URL-safe Base64 without padding.
///
/// ```lua
/// -- API Signature
/// aip.uuid.new_v4_b64u(): string
/// ```
///
/// ### Returns
///
/// `string`: The URL-safe Base64 encoded (no padding) UUIDv4 string.
///
/// ### Example
///
/// ```lua
/// local id_v4_b64u = aip.uuid.new_v4_b64u()
/// print(id_v4_b64u)
/// ```
fn lua_new_v4_b64url_nopad(_lua: &Lua, (): ()) -> mlua::Result<String> {
	Ok(uuid_extra::new_v4_b64url_nopad())
}

/// ## Lua Documentation aip.uuid.new_v4_b58
///
/// Generates a new UUID version 4 and encodes it using Base58.
///
/// ```lua
/// -- API Signature
/// aip.uuid.new_v4_b58(): string
/// ```
///
/// ### Returns
///
/// `string`: The Base58 encoded UUIDv4 string.
///
/// ### Example
///
/// ```lua
/// local id_v4_b58 = aip.uuid.new_v4_b58()
/// print(id_v4_b58)
/// ```
fn lua_new_v4_b58(_lua: &Lua, (): ()) -> mlua::Result<String> {
	Ok(uuid_extra::new_v4_b58())
}

/// ## Lua Documentation aip.uuid.new_v7_b64
///
/// Generates a new UUID version 7 and encodes it using standard Base64.
///
/// ```lua
/// -- API Signature
/// aip.uuid.new_v7_b64(): string
/// ```
///
/// ### Returns
///
/// `string`: The Base64 encoded UUIDv7 string.
///
/// ### Example
///
/// ```lua
/// local id_v7_b64 = aip.uuid.new_v7_b64()
/// print(id_v7_b64)
/// ```
fn lua_new_v7_b64(_lua: &Lua, (): ()) -> mlua::Result<String> {
	Ok(uuid_extra::new_v7_b64())
}

/// ## Lua Documentation aip.uuid.new_v7_b64u
///
/// Generates a new UUID version 7 and encodes it using URL-safe Base64 without padding.
///
/// ```lua
/// -- API Signature
/// aip.uuid.new_v7_b64u(): string
/// ```
///
/// ### Returns
///
/// `string`: The URL-safe Base64 encoded (no padding) UUIDv7 string.
///
/// ### Example
///
/// ```lua
/// local id_v7_b64u = aip.uuid.new_v7_b64u()
/// print(id_v7_b64u)
/// ```
fn lua_new_v7_b64url_nopad(_lua: &Lua, (): ()) -> mlua::Result<String> {
	Ok(uuid_extra::new_v7_b64url_nopad())
}

/// ## Lua Documentation aip.uuid.new_v7_b58
///
/// Generates a new UUID version 7 and encodes it using Base58.
///
/// ```lua
/// -- API Signature
/// aip.uuid.new_v7_b58(): string
/// ```
///
/// ### Returns
///
/// `string`: The Base58 encoded UUIDv7 string.
///
/// ### Example
///
/// ```lua
/// local id_v7_b58 = aip.uuid.new_v7_b58()
/// print(id_v7_b58)
/// ```
fn lua_new_v7_b58(_lua: &Lua, (): ()) -> mlua::Result<String> {
	Ok(uuid_extra::new_v7_b58())
}

/// ## Lua Documentation aip.uuid.to_time_epoch_ms
///
/// Converts a timestamped UUID string (V1, V6, V7) to milliseconds since Unix epoch.
/// Returns `nil` if the input is `nil`, not a valid UUID string, or if the UUID type
/// does not contain an extractable timestamp (e.g., V4).
///
/// ```lua
/// -- API Signature
/// aip.uuid.to_time_epoch_ms(value: string | nil): integer | nil
/// ```
///
/// ### Arguments
///
/// - `value: string | nil`: The UUID string or `nil`.
///
/// ### Returns
///
/// `integer | nil`: Milliseconds since Unix epoch, or `nil`.
///
/// ### Example
///
/// ```lua
/// local v7_uuid_str = aip.uuid.new_v7()
/// local millis = aip.uuid.to_time_epoch_ms(v7_uuid_str)
/// if millis then
///   print("Timestamp in ms: " .. millis)
/// else
///   print("Could not extract timestamp.")
/// end
///
/// local v4_uuid_str = aip.uuid.new_v4()
/// local millis_v4 = aip.uuid.to_time_epoch_ms(v4_uuid_str)
/// -- millis_v4 will be nil
///
/// local invalid_millis = aip.uuid.to_time_epoch_ms("not-a-uuid")
/// -- invalid_millis will be nil
///
/// local nil_millis = aip.uuid.to_time_epoch_ms(nil)
/// -- nil_millis will be nil
/// ```
fn lua_to_time_epoch_ms(_lua: &Lua, value: Value) -> mlua::Result<Value> {
	let Some(uuid_str) = into_option_string(value, "aip.uuid.to_time_epoch_ms")? else {
		return Ok(Value::Nil);
	};

	match Uuid::parse_str(&uuid_str) {
		Ok(uuid) => {
			if let Some(timestamp) = uuid.get_timestamp() {
				let (secs, nanos) = timestamp.to_unix();
				let millis = (secs as i64 * 1000) + (nanos as i64 / 1_000_000);
				Ok(Value::Integer(millis))
			} else {
				// UUID does not have an extractable timestamp (e.g., V4)
				Ok(Value::Nil)
			}
		}
		Err(_) => {
			// Invalid UUID string
			Ok(Value::Nil)
		}
	}
}

// endregion: --- Lua Functions

// endregion: --- Lua Interface

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{eval_lua, setup_lua};
	use uuid::Uuid;

	// region:    --- Support
	fn is_base64_url_chars(s: &str) -> bool {
		s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
	}

	fn is_base64_standard_chars(s: &str) -> bool {
		s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
	}

	fn is_base58_chars(s: &str) -> bool {
		const BASE58_CHARS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
		s.chars().all(|c| BASE58_CHARS.contains(c))
	}
	// endregion: --- Support

	#[tokio::test]
	async fn test_lua_aip_uuid_new_v4_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;

		// -- Exec
		let res_new = eval_lua(&lua, "return aip.uuid.new()")?;
		let res_new_v4 = eval_lua(&lua, "return aip.uuid.new_v4()")?;

		// -- Check
		let uuid_str_new = res_new.as_str().ok_or("aip.uuid.new() should return a string")?;
		let parsed_uuid_new = Uuid::parse_str(uuid_str_new)?;
		assert_eq!(
			parsed_uuid_new.get_version_num(),
			4,
			"UUID from new() should be version 4"
		);

		let uuid_str_new_v4 = res_new_v4.as_str().ok_or("aip.uuid.new_v4() should return a string")?;
		let parsed_uuid_new_v4 = Uuid::parse_str(uuid_str_new_v4)?;
		assert_eq!(
			parsed_uuid_new_v4.get_version_num(),
			4,
			"UUID from new_v4() should be version 4"
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_new_v7_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;

		// -- Exec
		let res = eval_lua(&lua, "return aip.uuid.new_v7()")?;

		// -- Check
		let uuid_str = res.as_str().ok_or("Result should be a string")?;
		let parsed_uuid = Uuid::parse_str(uuid_str)?;
		assert_eq!(parsed_uuid.get_version_num(), 7, "UUID should be version 7");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_new_v4_b64_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;

		// -- Exec
		let res = eval_lua(&lua, "return aip.uuid.new_v4_b64()")?;

		// -- Check
		let b64_str = res.as_str().ok_or("Result should be a string")?;
		assert!(
			is_base64_standard_chars(b64_str),
			"String should contain only Base64 standard characters (possibly with padding)"
		);
		// Standard Base64 of 16 bytes is typically 24 chars if padded with '=='
		// or 22 if unpadded (not standard for this function, new_v4_b64u is for unpadded)
		assert!(
			(22..=24).contains(&b64_str.len()),
			"Base64 string length for UUIDv4 should be between 22 and 24. Got: {}",
			b64_str.len()
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_new_v4_b64u_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;

		// -- Exec
		let res = eval_lua(&lua, "return aip.uuid.new_v4_b64u()")?;

		// -- Check
		let b64u_str = res.as_str().ok_or("Result should be a string")?;
		assert!(
			is_base64_url_chars(b64u_str),
			"String should contain only Base64 URL safe characters"
		);
		assert_eq!(
			b64u_str.len(),
			22,
			"Base64 URL (no pad) string length for UUID should be 22"
		);
		assert!(!b64u_str.contains('='), "String should not contain padding");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_new_v4_b58_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;

		// -- Exec
		let res = eval_lua(&lua, "return aip.uuid.new_v4_b58()")?;

		// -- Check
		let b58_str = res.as_str().ok_or("Result should be a string")?;
		assert!(is_base58_chars(b58_str), "String should contain only Base58 characters");
		// Base58 of 16 bytes is usually around 22 characters
		assert!(
			(21..=23).contains(&b58_str.len()),
			"Base58 string length for UUIDv4 should be around 21-23. Got: {}",
			b58_str.len()
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_new_v7_b64_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;

		// -- Exec
		let res = eval_lua(&lua, "return aip.uuid.new_v7_b64()")?;

		// -- Check
		let b64_str = res.as_str().ok_or("Result should be a string")?;
		assert!(
			is_base64_standard_chars(b64_str),
			"String should contain only Base64 standard characters"
		);
		assert!(
			(22..=24).contains(&b64_str.len()),
			"Base64 string length for UUIDv7 should be between 22 and 24. Got: {}",
			b64_str.len()
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_new_v7_b64u_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;

		// -- Exec
		let res = eval_lua(&lua, "return aip.uuid.new_v7_b64u()")?;

		// -- Check
		let b64u_str = res.as_str().ok_or("Result should be a string")?;
		assert!(
			is_base64_url_chars(b64u_str),
			"String should contain only Base64 URL safe characters"
		);
		assert_eq!(
			b64u_str.len(),
			22,
			"Base64 URL (no pad) string length for UUIDv7 should be 22"
		);
		assert!(!b64u_str.contains('='), "String should not contain padding");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_new_v7_b58_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;

		// -- Exec
		let res = eval_lua(&lua, "return aip.uuid.new_v7_b58()")?;

		// -- Check
		let b58_str = res.as_str().ok_or("Result should be a string")?;
		assert!(is_base58_chars(b58_str), "String should contain only Base58 characters");
		assert!(
			(21..=23).contains(&b58_str.len()),
			"Base58 string length for UUIDv7 should be around 21-23. Got: {}",
			b58_str.len()
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_to_time_epoch_ms_nil_input() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;
		let script = "return aip.uuid.to_time_epoch_ms(nil)";

		// -- Exec
		let result_val = eval_lua(&lua, script)?;

		// -- Check
		assert!(result_val.is_null(), "Expected nil for nil input");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_to_time_epoch_ms_invalid_uuid_string() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;
		let script = "return aip.uuid.to_time_epoch_ms('not-a-valid-uuid')";

		// -- Exec
		let result_val = eval_lua(&lua, script)?;

		// -- Check
		assert!(result_val.is_null(), "Expected nil for invalid UUID string");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_to_time_epoch_ms_v4_uuid() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;
		let v4_uuid_str = uuid_extra::new_v4().to_string();
		let script = format!("return aip.uuid.to_time_epoch_ms('{}')", v4_uuid_str);

		// -- Exec
		let result_val = eval_lua(&lua, &script)?;

		// -- Check
		assert!(
			result_val.is_null(),
			"Expected nil for V4 UUID as it has no extractable timestamp"
		);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_aip_uuid_to_time_epoch_ms_v7_uuid() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(init_module, "uuid")?;

		// Generate a V7 UUID and get its expected millisecond timestamp
		let v7_uuid = uuid_extra::new_v7(); // Uses current time
		let v7_uuid_str = v7_uuid.to_string();
		let ts_opt = v7_uuid.get_timestamp();
		let expected_millis = ts_opt
			.map(|ts| {
				let (secs, nanos) = ts.to_unix();
				(secs as i64 * 1000) + (nanos as i64 / 1_000_000)
			})
			.ok_or("Failed to get timestamp from generated V7 UUID")?;

		let script = format!("return aip.uuid.to_time_epoch_ms('{}')", v7_uuid_str);

		// -- Exec
		let result_val = eval_lua(&lua, &script)?;

		// -- Check
		let actual_millis = result_val.as_i64().ok_or("Result should be an integer (milliseconds)")?;
		assert_eq!(actual_millis, expected_millis, "Timestamp mismatch for V7 UUID");
		Ok(())
	}
}

// endregion: --- Tests
