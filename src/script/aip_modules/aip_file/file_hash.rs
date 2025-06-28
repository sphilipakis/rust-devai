//! Defines file hashing functions for the `aip.file` Lua module.
//!
//! This module provides functions to compute SHA256, SHA512, and BLAKE3 hashes
//! of file contents, with results available in various encodings (hex, Base64,
//! Base64URL, Base58).
//!
//! ---
//!
//! ## Lua documentation for `aip.file` hashing functions
//!
//! ### Common Behavior
//!
//! - All `path` arguments are resolved relative to the workspace root.
//! - Functions return the hash as a string on success.
//! - Errors (e.g., file not found, I/O issues) result in a Lua error.
//!
//! ### Functions
//!
//! **SHA256:**
//! - `aip.file.hash_sha256(path: string): string` - SHA256 hash, hex encoded.
//! - `aip.file.hash_sha256_b64(path: string): string` - SHA256 hash, Base64 encoded.
//! - `aip.file.hash_sha256_b64u(path: string): string` - SHA256 hash, Base64URL (no padding) encoded.
//! - `aip.file.hash_sha256_b58u(path: string): string` - SHA256 hash, Base58 encoded (naturally URL-safe).
//!
//! **SHA512:**
//! - `aip.file.hash_sha512(path: string): string` - SHA512 hash, hex encoded.
//! - `aip.file.hash_sha512_b64(path: string): string` - SHA512 hash, Base64 encoded.
//! - `aip.file.hash_sha512_b64u(path: string): string` - SHA512 hash, Base64URL (no padding) encoded.
//! - `aip.file.hash_sha512_b58u(path: string): string` - SHA512 hash, Base58 encoded (naturally URL-safe).
//!
//! **BLAKE3:**
//! - `aip.file.hash_blake3(path: string): string` - BLAKE3 hash, hex encoded.
//! - `aip.file.hash_blake3_b64(path: string): string` - BLAKE3 hash, Base64 encoded.
//! - `aip.file.hash_blake3_b64u(path: string): string` - BLAKE3 hash, Base64URL (no padding) encoded.
//! - `aip.file.hash_blake3_b58u(path: string): string` - BLAKE3 hash, Base58 encoded (naturally URL-safe).

use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::support::files::{
	hash_file_b58 as blake3_hash_file_b58, hash_file_b64 as blake3_hash_file_b64,
	hash_file_b64u as blake3_hash_file_b64u, hash_file_hex as blake3_hash_file_hex, hash_file_sha256_b58,
	hash_file_sha256_b64, hash_file_sha256_b64u, hash_file_sha256_hex, hash_file_sha512_b58, hash_file_sha512_b64,
	hash_file_sha512_b64u, hash_file_sha512_hex,
};
use mlua::{IntoLua, Lua, Value};

// region:    --- SHA256 Hashing Functions

/// ## Lua Documentation
///
/// Computes the SHA256 hash of a file and returns it as a hex-encoded string.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_sha256(path: string): string
/// ```
///
/// ### Example
///
/// ```lua
/// local hex_hash = aip.file.hash_sha256("my_document.txt")
/// print(hex_hash) -- e.g., "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"
/// ```
pub(super) fn file_hash_sha256(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = hash_file_sha256_hex(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_sha256 - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

/// ## Lua Documentation
///
/// Computes the SHA256 hash of a file and returns it as a Base64-encoded string.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_sha256_b64(path: string): string
/// ```
pub(super) fn file_hash_sha256_b64(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = hash_file_sha256_b64(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_sha256_b64 - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

/// ## Lua Documentation
///
/// Computes the SHA256 hash of a file and returns it as a Base64URL-encoded (no padding) string.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_sha256_b64u(path: string): string
/// ```
pub(super) fn file_hash_sha256_b64u(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = hash_file_sha256_b64u(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_sha256_b64u - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

/// ## Lua Documentation
///
/// Computes the SHA256 hash of a file and returns it as a Base58-encoded string.
/// Base58 is naturally URL-safe.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_sha256_b58u(path: string): string
/// ```
pub(super) fn file_hash_sha256_b58u(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = hash_file_sha256_b58(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_sha256_b58u - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

// endregion: --- SHA256 Hashing Functions

// region:    --- SHA512 Hashing Functions

/// ## Lua Documentation
///
/// Computes the SHA512 hash of a file and returns it as a hex-encoded string.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_sha512(path: string): string
/// ```
pub(super) fn file_hash_sha512(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = hash_file_sha512_hex(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_sha512 - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

/// ## Lua Documentation
///
/// Computes the SHA512 hash of a file and returns it as a Base64-encoded string.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_sha512_b64(path: string): string
/// ```
pub(super) fn file_hash_sha512_b64(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = hash_file_sha512_b64(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_sha512_b64 - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

/// ## Lua Documentation
///
/// Computes the SHA512 hash of a file and returns it as a Base64URL-encoded (no padding) string.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_sha512_b64u(path: string): string
/// ```
pub(super) fn file_hash_sha512_b64u(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = hash_file_sha512_b64u(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_sha512_b64u - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

/// ## Lua Documentation
///
/// Computes the SHA512 hash of a file and returns it as a Base58-encoded string.
/// Base58 is naturally URL-safe.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_sha512_b58u(path: string): string
/// ```
pub(super) fn file_hash_sha512_b58u(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = hash_file_sha512_b58(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_sha512_b58u - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

// endregion: --- SHA512 Hashing Functions

// region:    --- BLAKE3 Hashing Functions

/// ## Lua Documentation
///
/// Computes the BLAKE3 hash of a file and returns it as a hex-encoded string.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_blake3(path: string): string
/// ```
pub(super) fn file_hash_blake3(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = blake3_hash_file_hex(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_blake3 - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

/// ## Lua Documentation
///
/// Computes the BLAKE3 hash of a file and returns it as a Base64-encoded string.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_blake3_b64(path: string): string
/// ```
pub(super) fn file_hash_blake3_b64(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = blake3_hash_file_b64(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_blake3_b64 - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

/// ## Lua Documentation
///
/// Computes the BLAKE3 hash of a file and returns it as a Base64URL-encoded (no padding) string.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_blake3_b64u(path: string): string
/// ```
pub(super) fn file_hash_blake3_b64u(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = blake3_hash_file_b64u(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_blake3_b64u - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

/// ## Lua Documentation
///
/// Computes the BLAKE3 hash of a file and returns it as a Base58-encoded string.
/// Base58 is naturally URL-safe.
///
/// ```lua
/// -- API Signature
/// aip.file.hash_blake3_b58u(path: string): string
/// ```
pub(super) fn file_hash_blake3_b58u(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	let hash_string = blake3_hash_file_b58(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.hash_blake3_b58u - Failed to hash file '{path}'.\nCause: {e}"
		))
	})?;
	hash_string.into_lua(lua)
}

// endregion: --- BLAKE3 Hashing Functions
