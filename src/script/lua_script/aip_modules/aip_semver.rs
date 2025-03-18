//! Defines the `semver` module, used in the lua engine
//!
//! ---
//!
//! ## Lua documentation
//! This module exposes functions for semantic versioning operations.
//!
//! ### Functions
//! * `aip.semver.compare(version1: string, operator: string, version2: string): boolean`
//! * `aip.semver.parse(version: string): table`
//! * `aip.semver.is_prerelease(version: string): boolean`
//! * `aip.semver.valid(version: string): boolean`

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
/// ```lua
/// aip.semver.compare(version1: string, operator: string, version2: string) -> boolean
/// ```
///
/// Compares two version strings using the specified operator.
/// Valid operators: "<", "<=", "=", "==", ">=", ">", "!=", "~=".
///
/// Special rules for comparing versions:
/// - If versions have different major/minor/patch but either has prerelease/build,
///   only the major/minor/patch is compared (ignoring prerelease/build).
/// - If both versions have the same major/minor/patch, prerelease versions are
///   considered less than non-prerelease versions.
///
/// Returns true if the comparison is true, false otherwise.
///
/// Examples:
/// ```lua
/// aip.semver.compare("1.2.3", ">", "1.2.0") -- true
/// aip.semver.compare("1.2.3", "<", "1.3.0") -- true
/// aip.semver.compare("0.6.7-WIP", "<", "0.6.8") -- true
/// aip.semver.compare("0.6.7-WIP", "!=", "0.6.7") -- true
/// ```
fn compare(_lua: &Lua, (version1, operator, version2): (String, String, String)) -> mlua::Result<bool> {
	let v1 =
		parse_version(&version1).map_err(|e| mlua::Error::runtime(format!("Invalid version '{}': {}", version1, e)))?;

	let v2 =
		parse_version(&version2).map_err(|e| mlua::Error::runtime(format!("Invalid version '{}': {}", version2, e)))?;

	let val = match operator.as_str() {
		"<" => v1.lt(&v2),
		"<=" => v1.le(&v2),
		"=" | "==" => v1 == v2,
		">=" => v1.ge(&v2),
		">" => v1.gt(&v2),
		"!=" | "~=" => v1 != v2,
		_ => return Err(mlua::Error::RuntimeError(format!("Invalid operator '{}'", operator))),
	};

	Ok(val)
}

/// ## Lua Documentation
/// ```lua
/// aip.semver.parse(version: string) -> table
/// ```
///
/// Parses a version string into its components.
/// Returns a table with the following fields:
/// - major: number
/// - minor: number
/// - patch: number
/// - prerelease: string or nil
/// - build: string or nil
fn parse(_lua: &Lua, version: String) -> mlua::Result<Table> {
	let v =
		parse_version(&version).map_err(|e| mlua::Error::runtime(format!("Invalid version '{}': {}", version, e)))?;

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
/// ```lua
/// aip.semver.is_prerelease(version: string) -> boolean
/// ```
///
/// Returns true if the version is a prerelease (has a prerelease component).
fn is_prerelease(_lua: &Lua, version: String) -> mlua::Result<bool> {
	let v =
		parse_version(&version).map_err(|e| mlua::Error::runtime(format!("Invalid version '{}': {}", version, e)))?;

	Ok(!v.pre.is_empty())
}

/// ## Lua Documentation
/// ```lua
/// aip.semver.valid(version: string) -> boolean
/// ```
///
/// Returns true if the version string is a valid semantic version.
fn valid(_lua: &Lua, version: String) -> mlua::Result<bool> {
	Ok(parse_version(&version).is_ok())
}

fn parse_version(version: &str) -> Result<Version> {
	let version = Version::parse(version).map_err(crate::Error::custom)?;
	Ok(version)
}

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::lua_script::aip_semver;

	#[tokio::test]
	async fn test_lua_semver_compare_basic() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_semver::init_module, "semver")?;

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
			let script = format!(r#"return aip.semver.compare("{}", "{}", "{}")"#, v1, op, v2);
			let result: bool = eval_lua(&lua, &script)?.as_bool().ok_or("should be bool")?;
			assert_eq!(
				result, expected,
				"Failed for compare(\"{}\", \"{}\", \"{}\"): expected {}, got {}",
				v1, op, v2, expected, result
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_semver_compare_with_prerelease() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_semver::init_module, "semver")?;

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
			let script = format!(r#"return aip.semver.compare("{}", "{}", "{}")"#, v1, op, v2);
			let result: bool = eval_lua(&lua, &script)?.as_bool().ok_or("should be bool")?;
			assert_eq!(
				result, expected,
				"Failed for compare(\"{}\", \"{}\", \"{}\"): expected {}, got {}",
				v1, op, v2, expected, result
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_semver_parse() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_semver::init_module, "semver")?;

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
		let lua = setup_lua(aip_semver::init_module, "semver")?;

		let test_cases = [
			("1.2.3", false),
			("1.2.3-beta", true),
			("0.6.7-WIP", true),
			("1.0.0+build.123", false),
			("1.0.0-alpha+build.123", true),
		];

		for (version, expected) in test_cases {
			let script = format!(r#"return aip.semver.is_prerelease("{}")"#, version);
			let result: bool = eval_lua(&lua, &script)?.as_bool().ok_or("should be bool")?;
			assert_eq!(
				result, expected,
				"Failed for is_prerelease(\"{}\"): expected {}, got {}",
				version, expected, result
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_semver_valid() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_semver::init_module, "semver")?;

		let test_cases = [
			("1.2.3", true),
			("1.2.3-beta", true),
			("0.6.7-WIP", true),
			("invalid", false),
			("1.0", false),
			("1.0.0+build.123", true),
		];

		for (version, expected) in test_cases {
			let script = format!(r#"return aip.semver.valid("{}")"#, version);
			let result: bool = eval_lua(&lua, &script)?.as_bool().ok_or("should be bool")?;
			assert_eq!(
				result, expected,
				"Failed for valid(\"{}\"): expected {}, got {}",
				version, expected, result
			);
		}

		Ok(())
	}
}
