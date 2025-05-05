//! Helper functions for JSON operations.

use crate::{Error, Result};
use serde_json::Value;

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
