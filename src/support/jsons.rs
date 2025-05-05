use crate::{Error, Result};
use serde_json::Value;
use std::io::BufRead;

pub fn into_values<T: serde::Serialize>(vals: Vec<T>) -> Result<Vec<Value>> {
	let inputs: Vec<Value> = vals
		.into_iter()
		.map(|v| serde_json::to_value(v).map_err(Error::custom))
		.collect::<Result<Vec<_>>>()?;

	Ok(inputs)
}

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
