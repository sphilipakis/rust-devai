//! Defines the `aip.hash` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.hash` module exposes functions for various hashing algorithms and encodings.
//!
//! ### Functions
//!
//! - `aip.hash.sha256(input: string): string` - SHA256 hash, hex-encoded.
//! - `aip.hash.sha256_b58(input: string): string` - SHA256 hash, Base58-encoded.
//! - `aip.hash.sha256_b64(input: string): string` - SHA256 hash, standard Base64-encoded.
//! - `aip.hash.sha256_b64u(input: string): string` - SHA256 hash, URL-safe Base64-encoded (no padding).
//! - `aip.hash.sha512(input: string): string` - SHA512 hash, hex-encoded.
//! - `aip.hash.sha512_b58(input: string): string` - SHA512 hash, Base58-encoded.
//! - `aip.hash.sha512_b64(input: string): string` - SHA512 hash, standard Base64-encoded.
//! - `aip.hash.sha512_b64u(input: string): string` - SHA512 hash, URL-safe Base64-encoded (no padding).
//! - `aip.hash.blake3(input: string): string` - Blake3 hash, hex-encoded.
//! - `aip.hash.blake3_b58(input: string): string` - Blake3 hash, Base58-encoded.
//! - `aip.hash.blake3_b64(input: string): string` - Blake3 hash, standard Base64-encoded.
//! - `aip.hash.blake3_b64u(input: string): string` - Blake3 hash, URL-safe Base64-encoded (no padding).

use crate::Result;
use crate::runtime::Runtime;
use mlua::{Lua, Table};
use sha2::{Digest, Sha256, Sha512};
// Added blake3 Hasher
use blake3::Hasher;

/// Initializes the `hash` Lua module.
///
/// Registers all hashing functions in the module table.
pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	table.set("sha256", lua.create_function(lua_sha256)?)?;
	table.set("sha256_b58", lua.create_function(lua_sha256_b58)?)?;
	table.set("sha256_b64", lua.create_function(lua_sha256_b64)?)?;
	table.set("sha256_b64u", lua.create_function(lua_sha256_b64u)?)?;

	table.set("sha512", lua.create_function(lua_sha512)?)?;
	table.set("sha512_b58", lua.create_function(lua_sha512_b58)?)?;
	table.set("sha512_b64", lua.create_function(lua_sha512_b64)?)?;
	table.set("sha512_b64u", lua.create_function(lua_sha512_b64u)?)?;

	// -- Blake3 functions
	table.set("blake3", lua.create_function(lua_blake3)?)?;
	table.set("blake3_b58", lua.create_function(lua_blake3_b58)?)?;
	table.set("blake3_b64", lua.create_function(lua_blake3_b64)?)?;
	table.set("blake3_b64u", lua.create_function(lua_blake3_b64u)?)?;

	Ok(table)
}

// region:    --- SHA256 Functions

/// ## Lua Documentation
///
/// `aip.hash.sha256(input: string): string`
///
/// Computes the SHA256 hash of the input string and returns it as a lowercase hex-encoded string.
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The SHA256 hash, hex-encoded.
///
/// ### Example
///
/// ```lua
/// local hex_hash = aip.hash.sha256("hello world")
/// -- hex_hash will be "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
/// print(hex_hash)
/// ```
fn lua_sha256(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Sha256::new();
	hasher.update(input.as_bytes());
	let result = hasher.finalize();
	Ok(hex::encode(result))
}

/// ## Lua Documentation
///
/// `aip.hash.sha256_b58(input: string): string`
///
/// Computes the SHA256 hash of the input string and returns it as a Base58-encoded string.
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The SHA256 hash, Base58-encoded.
///
/// ### Example
///
/// ```lua
/// local b58_hash = aip.hash.sha256_b58("hello world")
/// -- b58_hash will be "CnKqR4x6r4nAv2iGk8DrZSSWp7n3W9xKRj69eZysS272"
/// print(b58_hash)
/// ```
fn lua_sha256_b58(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Sha256::new();
	hasher.update(input.as_bytes());
	let result = hasher.finalize();
	Ok(bs58::encode(result).into_string())
}

/// ## Lua Documentation
///
/// `aip.hash.sha256_b64(input: string): string`
///
/// Computes the SHA256 hash of the input string and returns it as a standard Base64-encoded string (RFC 4648).
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The SHA256 hash, standard Base64-encoded.
///
/// ### Example
///
/// ```lua
/// local b64_hash = aip.hash.sha256_b64("hello world")
/// -- b64_hash will be "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek="
/// print(b64_hash)
/// ```
fn lua_sha256_b64(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Sha256::new();
	hasher.update(input.as_bytes());
	let result = hasher.finalize();
	Ok(base64::encode_engine(
		result,
		&base64::engine::general_purpose::STANDARD,
	))
}

/// ## Lua Documentation
///
/// `aip.hash.sha256_b64u(input: string): string`
///
/// Computes the SHA256 hash of the input string and returns it as a URL-safe Base64-encoded string (RFC 4648, section 5), without padding.
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The SHA256 hash, URL-safe Base64-encoded without padding.
///
/// ### Example
///
/// ```lua
/// local b64u_hash = aip.hash.sha256_b64u("hello world")
/// -- b64u_hash will be "uU0nuZNNPgilLlLX2n2r-sSE7-N6U4DukIj3rOLvzek"
/// print(b64u_hash)
/// ```
fn lua_sha256_b64u(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Sha256::new();
	hasher.update(input.as_bytes());
	let result = hasher.finalize();
	Ok(base64::encode_engine(
		result,
		&base64::engine::general_purpose::URL_SAFE_NO_PAD,
	))
}

// endregion: --- SHA256 Functions

// region:    --- SHA512 Functions

/// ## Lua Documentation
///
/// `aip.hash.sha512(input: string): string`
///
/// Computes the SHA512 hash of the input string and returns it as a lowercase hex-encoded string.
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The SHA512 hash, hex-encoded.
///
/// ### Example
///
/// ```lua
/// local hex_hash = aip.hash.sha512("hello world")
/// -- hex_hash will be "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
/// print(hex_hash)
/// ```
fn lua_sha512(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Sha512::new();
	hasher.update(input.as_bytes());
	let result = hasher.finalize();
	Ok(hex::encode(result))
}

/// ## Lua Documentation
///
/// `aip.hash.sha512_b58(input: string): string`
///
/// Computes the SHA512 hash of the input string and returns it as a Base58-encoded string.
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The SHA512 hash, Base58-encoded.
///
/// ### Example
///
/// ```lua
/// local b58_hash = aip.hash.sha512_b58("hello world")
/// -- b58_hash will be "yP4cqy7jmaRDzC2bmcGNZkuQb3VdftMk6YH7ynQ2Qw4zktKsyA9fk52xghNQNAdkpF9iFmFkKh2bNVG4kDWhsok"
/// print(b58_hash)
/// ```
fn lua_sha512_b58(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Sha512::new();
	hasher.update(input.as_bytes());
	let result = hasher.finalize();
	Ok(bs58::encode(result).into_string())
}

/// ## Lua Documentation
///
/// `aip.hash.sha512_b64(input: string): string`
///
/// Computes the SHA512 hash of the input string and returns it as a standard Base64-encoded string (RFC 4648).
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The SHA512 hash, standard Base64-encoded.
///
/// ### Example
///
/// ```lua
/// local b64_hash = aip.hash.sha512_b64("hello world")
/// -- b64_hash will be "MJ7MSJwS1utMxA9QyQLytNDtd+5RGnx6m808qG1M2G+YndNbxf9JlnDaNCVbRbDP2DDoH2Bdz33FVC6TrpzXbw=="
/// print(b64_hash)
/// ```
fn lua_sha512_b64(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Sha512::new();
	hasher.update(input.as_bytes());
	let result = hasher.finalize();
	Ok(base64::encode_engine(
		result,
		&base64::engine::general_purpose::STANDARD,
	))
}

/// ## Lua Documentation
///
/// `aip.hash.sha512_b64u(input: string): string`
///
/// Computes the SHA512 hash of the input string and returns it as a URL-safe Base64-encoded string (RFC 4648, section 5), without padding.
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The SHA512 hash, URL-safe Base64-encoded without padding.
///
/// ### Example
///
/// ```lua
/// local b64u_hash = aip.hash.sha512_b64u("hello world")
/// -- b64u_hash will be "MJ7MSJwS1utMxA9QyQLytNDtd-5RGnx6m808qG1M2G-YndNbxf9JlnDaNCVbRbDP2DDoH2Bdz33FVC6TrpzXbw"
/// print(b64u_hash)
/// ```
fn lua_sha512_b64u(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Sha512::new();
	hasher.update(input.as_bytes());
	let result = hasher.finalize();
	Ok(base64::encode_engine(
		result,
		&base64::engine::general_purpose::URL_SAFE_NO_PAD,
	))
}

// endregion: --- SHA512 Functions

// region:    --- Blake3 Functions

/// ## Lua Documentation
///
/// `aip.hash.blake3(input: string): string`
///
/// Computes the Blake3 hash of the input string and returns it as a lowercase hex-encoded string.
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The Blake3 hash, hex-encoded.
///
/// ### Example
///
/// ```lua
/// local hex_hash = aip.hash.blake3("hello world")
/// -- hex_hash will be "d74981efa70a0c5807681807da0623991e10b7919751410380587aaf1857c569"
/// print(hex_hash)
/// ```
fn lua_blake3(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Hasher::new();
	hasher.update(input.as_bytes());
	let hash_bytes = hasher.finalize();
	Ok(hash_bytes.to_hex().to_string())
}

/// ## Lua Documentation
///
/// `aip.hash.blake3_b58(input: string): string`
///
/// Computes the Blake3 hash of the input string and returns it as a Base58-encoded string.
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The Blake3 hash, Base58-encoded.
///
/// ### Example
///
/// ```lua
/// local b58_hash = aip.hash.blake3_b58("hello world")
/// -- b58_hash will be "F8Pqk8Yk1FkY2dj4XmG1gKkE2H4QJ3uW1vP9qL7zRwG9"
/// print(b58_hash)
/// ```
fn lua_blake3_b58(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Hasher::new();
	hasher.update(input.as_bytes());
	let hash_bytes = hasher.finalize();
	Ok(bs58::encode(hash_bytes.as_bytes()).into_string())
}

/// ## Lua Documentation
///
/// `aip.hash.blake3_b64(input: string): string`
///
/// Computes the Blake3 hash of the input string and returns it as a standard Base64-encoded string (RFC 4648).
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The Blake3 hash, standard Base64-encoded.
///
/// ### Example
///
/// ```lua
/// local b64_hash = aip.hash.blake3_b64("hello world")
/// -- b64_hash will be "10mB76cKDFgHaBgH2gYjmx4Qt5GXVUEGgFd6rxhXxWk="
/// print(b64_hash)
/// ```
fn lua_blake3_b64(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Hasher::new();
	hasher.update(input.as_bytes());
	let hash_bytes = hasher.finalize();
	Ok(base64::encode_engine(
		hash_bytes.as_bytes(),
		&base64::engine::general_purpose::STANDARD,
	))
}

/// ## Lua Documentation
///
/// `aip.hash.blake3_b64u(input: string): string`
///
/// Computes the Blake3 hash of the input string and returns it as a URL-safe Base64-encoded string (RFC 4648, section 5), without padding.
///
/// ### Arguments
///
/// - `input: string`: The string to hash.
///
/// ### Returns
///
/// `string`: The Blake3 hash, URL-safe Base64-encoded without padding.
///
/// ### Example
///
/// ```lua
/// local b64u_hash = aip.hash.blake3_b64u("hello world")
/// -- b64u_hash will be "10mB76cKDFgHaBgH2gYjmx4Qt5GXVUEGgFd6rxhXxWk"
/// print(b64u_hash)
/// ```
fn lua_blake3_b64u(_lua: &Lua, input: String) -> mlua::Result<String> {
	let mut hasher = Hasher::new();
	hasher.update(input.as_bytes());
	let hash_bytes = hasher.finalize();
	Ok(base64::encode_engine(
		hash_bytes.as_bytes(),
		&base64::engine::general_purpose::URL_SAFE_NO_PAD,
	))
}

// endregion: --- Blake3 Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_hash;

	const TEST_INPUT: &str = "hello world";

	#[tokio::test]
	async fn test_lua_hash_sha256_hex() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.sha256("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_sha256_b58() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "CnKqR4x6r4nAv2iGk8DrZSSWp7n3W9xKRj69eZysS272"; // Corrected expected value

		// -- Exec
		let lua_code = format!(r#"return aip.hash.sha256_b58("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_sha256_b64() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.sha256_b64("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_sha256_b64u() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "uU0nuZNNPgilLlLX2n2r-sSE7-N6U4DukIj3rOLvzek";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.sha256_b64u("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_sha512_hex() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.sha512("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_sha512_b58() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "yP4cqy7jmaRDzC2bmcGNZkuQb3VdftMk6YH7ynQ2Qw4zktKsyA9fk52xghNQNAdkpF9iFmFkKh2bNVG4kDWhsok";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.sha512_b58("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_sha512_b64() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "MJ7MSJwS1utMxA9QyQLytNDtd+5RGnx6m808qG1M2G+YndNbxf9JlnDaNCVbRbDP2DDoH2Bdz33FVC6TrpzXbw==";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.sha512_b64("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_sha512_b64u() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "MJ7MSJwS1utMxA9QyQLytNDtd-5RGnx6m808qG1M2G-YndNbxf9JlnDaNCVbRbDP2DDoH2Bdz33FVC6TrpzXbw";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.sha512_b64u("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	// region:    --- Blake3 Tests
	#[tokio::test]
	async fn test_lua_hash_blake3_hex() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.blake3("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_blake3_b58() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "FVPfbg9bK7mj7jnaSRXhuVcVakkXcjMPgSwxmauUofYf";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.blake3_b58("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_blake3_b64() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "10mB76cKDIgLjYwZhdB128v2ebmaX5kU5ar5a4ManiQ=";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.blake3_b64("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hash_blake3_b64u() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_hash::init_module, "hash")?;
		let expected = "10mB76cKDIgLjYwZhdB128v2ebmaX5kU5ar5a4ManiQ";

		// -- Exec
		let lua_code = format!(r#"return aip.hash.blake3_b64u("{}")"#, TEST_INPUT);
		let res = eval_lua(&lua, &lua_code)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Result should be a string")?, expected);
		Ok(())
	}
	// endregion: --- Blake3 Tests
}

// endregion: --- Tests
