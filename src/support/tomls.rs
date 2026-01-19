//! Crate utility for toml
//!
//! Note: The goal is that all get serialized to serded_json as this is the cannonical format for now.

use crate::{Error, Result};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

pub fn parse_toml_into_json(toml_content: &str) -> Result<JsonValue> {
	// Parse the TOML string into a TOML value
	let toml_value: TomlValue = toml::from_str(toml_content)?;

	// Convert the TOML value to a serde_json::Value
	let json_value = serde_json::to_value(toml_value)?;

	Ok(json_value)
}

/// Stringify a `serde_json::Value` into a TOML string.
pub fn stringify_json_value_to_toml_string(json_value: &JsonValue) -> Result<String> {
	toml::to_string(json_value).map_err(|err| Error::cc(format!("Cannot stringify {json_value:?}"), err))
}
