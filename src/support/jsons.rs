//! Helper functions for JSON operations.

use crate::{Error, Result};
use serde_json::Value;
use std::io::BufRead;

/// Converts a `Vec<T>` where `T` is serializable into a `Result<Vec<Value>>`.
///
/// Serializes each item in the input vector into a `serde_json::Value`.
///
/// # Arguments
///
/// * `vals` - A `Vec` of items implementing `serde::Serialize`.
///
/// # Returns
///
/// Returns `Ok(Vec<Value>)` on success, or an `Error` if serialization fails.
pub fn into_values<T: serde::Serialize>(vals: Vec<T>) -> Result<Vec<Value>> {
	let inputs: Vec<Value> = vals
		.into_iter()
		.map(|v| serde_json::to_value(v).map_err(Error::custom))
		.collect::<Result<Vec<_>>>()?;

	Ok(inputs)
}

/// Parses newline-delimited JSON (NDJSON) from a `BufRead` reader.
///
/// Reads lines from the reader, parses each non-empty line as a JSON object,
/// and collects them into a `serde_json::Value::Array`.
///
/// # Arguments
///
/// * `reader` - Any type implementing `BufRead`.
///
/// # Returns
///
/// Returns `Ok(Value::Array)` containing the parsed JSON values on success,
/// or an `Error` if reading lines or parsing JSON fails.
pub fn parse_ndjson_from_reader<R: BufRead>(reader: R) -> Result<Value> {
	let mut values = Vec::new();

	for (index, line_result) in reader.lines().enumerate() {
		let line = line_result.map_err(|e| {
			Error::from(format!(
				"aip.file.load_ndjson - Failed to read line {}. Cause: {}",
				index + 1,
				e
			))
		})?;

		if line.trim().is_empty() {
			continue;
		}

		let json_value: Value = serde_json::from_str(&line).map_err(|e| {
			Error::from(format!(
				"aip.file.load_ndjson - Failed to parse JSON on line {}. Cause: {}",
				index + 1,
				e
			))
		})?;

		values.push(json_value);
	}

	Ok(Value::Array(values))
}
