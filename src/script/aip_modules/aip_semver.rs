//! Defines the `semver` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This module exposes functions for semantic versioning operations, conforming to the
//! [Semantic Versioning 2.0.0 specification](https://semver.org/).
//!
//! ### Functions
//!
//! - `aip.semver.compare(version1: string, operator: string, version2: string): boolean`
//!   Compares two version strings using the specified operator.
//! - `aip.semver.parse(version: string): {major: number, minor: number, patch: number, prerelease: string | nil, build: string | nil}`
//!   Parses a version string into its components.
//! - `aip.semver.is_prerelease(version: string): boolean`
//!   Returns `true` if the version is a prerelease (has a prerelease component).
//! - `aip.semver.valid(version: string): boolean`
//!   Returns `true` if the version string is a valid semantic version.
//!
//! ---

use crate::Result;
use crate::runtime::Runtime;
use mlua::{Lua, Table, Value};
use semver::Version;

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	table.set("compare", lua.create_function(compare)?)?;
	table.set("parse", lua.create_function(parse)?)?;
	table.set("is_prerelease", lua.create_function(is_prerelease)?)?;
	table.set("valid", lua.create_function(valid)?)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Compares two version strings using the specified operator.
///
/// ```lua
/// -- API Signature
/// aip.semver.compare(version1: string, operator: string, version2: string): boolean
/// ```
///
/// Compares `version1` and `version2` based on the provided `operator`.
/// Comparisons follow the Semantic Versioning 2.0.0 specification rules,
/// including prerelease handling (prerelease versions are lower than release versions).
///
/// ### Arguments
///
/// - `version1: string`: The first version string to compare.
/// - `operator: string`: The comparison operator. Valid operators are:
///   - `"<"`: Less than
///   - `"<="`: Less than or equal to
///   - `"="`, `"=="`: Equal to
///   - `">="`: Greater than or equal to
///   - `">"`: Greater than
///   - `"!="`, `"~="`: Not equal to (Note: `"~="` is an alias for `"!="` in this module for flexibility)
/// - `version2: string`: The second version string to compare against.
///
/// ### Returns
///
/// `boolean`: `true` if the comparison result is true, `false` otherwise.
///
/// ### Example
///
/// ```lua
/// print(aip.semver.compare("1.2.3", ">", "1.2.0"))     -- Output: true
/// print(aip.semver.compare("1.2.3", "<", "1.3.0"))     -- Output: true
/// print(aip.semver.compare("0.6.7-WIP", "<", "0.6.8")) -- Output: true
/// print(aip.semver.compare("0.6.7-WIP", "!=", "0.6.7"))-- Output: true
/// print(aip.semver.compare("1.0.0-alpha", "<", "1.0.0")) -- Output: true (prerelease < release)
/// print(aip.semver.compare("1.0.0+build", "==", "1.0.0")) -- Output: true (build metadata is ignored in comparisons)
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - `operator` is not one of the valid comparison operators.
/// - `version1` or `version2` are not valid semantic version strings.
fn compare(_lua: &Lua, (version1, operator, version2): (String, String, String)) -> mlua::Result<bool> {
	let v1 =
		parse_version(&version1).map_err(|e| mlua::Error::runtime(format!("Invalid version '{version1}': {e}")))?;

	let v2 =
		parse_version(&version2).map_err(|e| mlua::Error::runtime(format!("Invalid version '{version2}': {e}")))?;

	let val = match operator.as_str() {
		"<" => v1.lt(&v2),
		"<=" => v1.le(&v2),
		"=" | "==" => v1 == v2,
		">=" => v1.ge(&v2),
		">" => v1.gt(&v2),
		"!=" | "~=" => v1 != v2,
		_ => return Err(mlua::Error::RuntimeError(format!("Invalid operator '{operator}'"))),
	};

	Ok(val)
}

/// ## Lua Documentation
///
/// Parses a version string into its components.
///
/// ```lua
/// -- API Signature
/// aip.semver.parse(version: string): {major: number, minor: number, patch: number, prerelease: string | nil, build: string | nil}
/// ```
///
/// Parses the provided semantic `version` string into a table containing its major, minor, patch,
/// prerelease, and build metadata components.
///
/// ### Arguments
///
/// - `version: string`: The semantic version string to parse (e.g., `"1.2.3-beta.1+build.123"`).
///
/// ### Returns
///
/// Returns a Lua table representing the parsed version components:
///
/// ```ts
/// {
///   major: number,         // The major version number.
///   minor: number,         // The minor version number.
///   patch: number,         // The patch version number.
///   prerelease: string | nil, // The prerelease string (e.g., "beta.1"), or nil if not present.
///   build: string | nil    // The build metadata string (e.g., "build.123"), or nil if not present.
/// }
/// ```
///
/// ### Example
///
/// ```lua
/// local result = aip.semver.parse("1.2.3-beta.1+build.123")
/// print("Major:", result.major)       -- Output: Major: 1
/// print("Minor:", result.minor)       -- Output: Minor: 2
/// print("Patch:", result.patch)       -- Output: Patch: 3
/// print("Prerelease:", result.prerelease) -- Output: Prerelease: beta.1
/// print("Build:", result.build)       -- Output: Build: build.123
///
/// local simple_ver = aip.semver.parse("2.0.0")
/// print("Prerelease:", simple_ver.prerelease) -- Output: Prerelease: nil
/// print("Build:", simple_ver.build)       -- Output: Build: nil
/// ```
///
/// ### Error
///
/// Returns an error if the `version` string is not a valid semantic version.
fn parse(_lua: &Lua, version: String) -> mlua::Result<Table> {
	let v = parse_version(&version).map_err(|e| mlua::Error::runtime(format!("Invalid version '{version}': {e}")))?;

	let table = _lua.create_table()?;
	table.set("major", v.major)?;
	table.set("minor", v.minor)?;
	table.set("patch", v.patch)?;

	if !v.pre.is_empty() {
		table.set("prerelease", v.pre.to_string())?;
	} else {
		table.set("prerelease", Value::Nil)?;
	}

	if !v.build.is_empty() {
		table.set("build", v.build.to_string())?;
	} else {
		table.set("build", Value::Nil)?;
	}

	Ok(table)
}

/// ## Lua Documentation
///
/// Returns `true` if the version is a prerelease (has a prerelease component).
///
/// ```lua
/// -- API Signature
/// aip.semver.is_prerelease(version: string): boolean
/// ```
///
/// Checks if the given semantic `version` string contains a prerelease component (e.g., `-alpha`, `-beta.1`, `-rc.2`).
///
/// ### Arguments
///
/// - `version: string`: The semantic version string to check.
///
/// ### Returns
///
/// `boolean`: `true` if the version has a non-empty prerelease component, `false` otherwise.
///
/// ### Example
///
/// ```lua
/// print(aip.semver.is_prerelease("1.2.3"))         -- Output: false
/// print(aip.semver.is_prerelease("1.2.3-beta"))      -- Output: true
/// print(aip.semver.is_prerelease("0.6.7-WIP"))      -- Output: true
/// print(aip.semver.is_prerelease("1.0.0+build.123")) -- Output: false (build metadata is not prerelease)
/// ```
///
/// ### Error
///
/// Returns an error if the `version` string is not a valid semantic version.
fn is_prerelease(_lua: &Lua, version: String) -> mlua::Result<bool> {
	let v = parse_version(&version).map_err(|e| mlua::Error::runtime(format!("Invalid version '{version}': {e}")))?;

	Ok(!v.pre.is_empty())
}

/// ## Lua Documentation
///
/// Returns `true` if the version string is a valid semantic version.
///
/// ```lua
/// -- API Signature
/// aip.semver.valid(version: string): boolean
/// ```
///
/// Checks if the provided `version` string conforms to the Semantic Versioning 2.0.0 specification.
///
/// ### Arguments
///
/// - `version: string`: The string to validate as a semantic version.
///
/// ### Returns
///
/// `boolean`: `true` if the string is a valid semantic version, `false` otherwise.
///
/// ### Example
///
/// ```lua
/// print(aip.semver.valid("1.2.3"))          -- Output: true
/// print(aip.semver.valid("1.2.3-alpha.1"))   -- Output: true
/// print(aip.semver.valid("1.0.0+build.456"))  -- Output: true
/// print(aip.semver.valid("invalid-version")) -- Output: false
/// print(aip.semver.valid("1.0"))           -- Output: false (missing patch)
/// ```
fn valid(_lua: &Lua, version: String) -> mlua::Result<bool> {
	Ok(parse_version(&version).is_ok())
}

/// Helper function to parse a version string using the `semver` crate.
fn parse_version(version: &str) -> Result<Version> {
	let version = Version::parse(version).map_err(crate::Error::custom)?;
	Ok(version)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_semver;

	#[tokio::test]
	async fn test_lua_semver_compare_basic() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_semver::init_module, "semver").await?;

		// Test cases: (version1, operator, version2, expected_result)
		let test_cases = [
			("1.2.3", ">", "1.2.0", true),
			("1.2.3", "<", "1.3.0", true),
			("1.2.3", "=", "1.2.3", true),
			("1.2.3", "==", "1.2.3", true),
			("1.2.3", ">=", "1.2.3", true),
			("1.2.3", "<=", "1.2.3", true),
			("1.2.3", ">", "1.2.3", false),
			("1.2.3", "<", "1.2.3", false),
			("1.2.3", "!=", "1.2.0", true),
			("1.2.3", "~=", "1.2.0", true),
			("0.6.12-WIP", ">", "0.6.9", true),
			("0.6.7-wip", "<", "0.6.8", true),
			("0.6.7-wip", "<", "0.6.7", true),
			("0.6.7-alpha", "<", "0.6.7", true),
			("0.6.7-alpha.1", "<", "0.6.7-alpha.2", true),
			("0.6.7-alpha.1", "<", "0.6.7-beta", true),
		];

		for (v1, op, v2, expected) in test_cases {
			let script = format!(r#"return aip.semver.compare("{v1}", "{op}", "{v2}")"#);
			let result: bool = eval_lua(&lua, &script)?.as_bool().ok_or("should be bool")?;
			assert_eq!(
				result, expected,
				"Failed for compare(\"{v1}\", \"{op}\", \"{v2}\"): expected {expected}, got {result}"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_semver_compare_with_prerelease() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_semver::init_module, "semver").await?;

		// Test cases specifically for prerelease version comparison rules
		let test_cases = [
			// When versions have different major/minor/patch and either has prerelease
			("1.2.3-alpha", ">", "1.2.0", true),  // Compares only major.minor.patch
			("1.2.0", "<", "1.2.3-alpha", true),  // Compares only major.minor.patch
			("1.0.0+build", "=", "1.0.0", false), // Build metadata should be ignored
			// When versions have same major/minor/patch, prerelease is considered
			("1.2.3-alpha", "<", "1.2.3", true), // Prerelease is less than non-prerelease
			("1.2.3-alpha", "<", "1.2.4", true), // Definitely <
			("1.2.3", ">", "1.2.3-alpha", true), // Non-prerelease is greater than prerelease
			// NotEq operators
			("1.2.3-alpha", "!=", "1.2.3", true), // Different due to prerelease
			("1.0.0+build", "!=", "1.0.0", true), // Build metadata should be ignored
			("1.2.3-alpha", "~=", "1.2.3", true), // Different due to prerelease
		];

		for (v1, op, v2, expected) in test_cases {
			let script = format!(r#"return aip.semver.compare("{v1}", "{op}", "{v2}")"#);
			let result: bool = eval_lua(&lua, &script)?.as_bool().ok_or("should be bool")?;
			assert_eq!(
				result, expected,
				"Failed for compare(\"{v1}\", \"{op}\", \"{v2}\"): expected {expected}, got {result}"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_semver_parse() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_semver::init_module, "semver").await?;

		let script = r#"
        local result = aip.semver.parse("1.2.3-beta.1+build.123")
        return {
            major = result.major,
            minor = result.minor,
            patch = result.patch,
            prerelease = result.prerelease,
            build = result.build
        }
        "#;

		let result = eval_lua(&lua, script)?;

		let json_result = serde_json::to_string(&result)?;
		let parsed: serde_json::Value = serde_json::from_str(&json_result)?;

		assert_eq!(parsed["major"], 1);
		assert_eq!(parsed["minor"], 2);
		assert_eq!(parsed["patch"], 3);
		assert_eq!(parsed["prerelease"], "beta.1");
		assert_eq!(parsed["build"], "build.123");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_semver_is_prerelease() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_semver::init_module, "semver").await?;

		let test_cases = [
			("1.2.3", false),
			("1.2.3-beta", true),
			("0.6.7-WIP", true),
			("1.0.0+build.123", false),
			("1.0.0-alpha+build.123", true),
		];

		for (version, expected) in test_cases {
			let script = format!(r#"return aip.semver.is_prerelease("{version}")"#);
			let result: bool = eval_lua(&lua, &script)?.as_bool().ok_or("should be bool")?;
			assert_eq!(
				result, expected,
				"Failed for is_prerelease(\"{version}\"): expected {expected}, got {result}"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_semver_valid() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_semver::init_module, "semver").await?;

		let test_cases = [
			("1.2.3", true),
			("1.2.3-beta", true),
			("0.6.7-WIP", true),
			("invalid", false),
			("1.0", false),
			("1.0.0+build.123", true),
		];

		for (version, expected) in test_cases {
			let script = format!(r#"return aip.semver.valid("{version}")"#);
			let result: bool = eval_lua(&lua, &script)?.as_bool().ok_or("should be bool")?;
			assert_eq!(
				result, expected,
				"Failed for valid(\"{version}\"): expected {expected}, got {result}"
			);
		}

		Ok(())
	}
}

// endregion: --- Tests
