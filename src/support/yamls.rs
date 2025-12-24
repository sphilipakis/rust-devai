use crate::{Error, Result};
use crate::types::YamlDocs;
use serde::Deserialize;
use serde_json::Value;

pub fn parse(content: &str) -> Result<YamlDocs> {
	let mut docs = Vec::new();
	for document in serde_yaml_ng::Deserializer::from_str(content) {
		let value = Value::deserialize(document)?;
		docs.push(value);
	}
	Ok(YamlDocs::new(docs))
}

pub fn stringify(value: &Value) -> Result<String> {
	serde_yaml_ng::to_string(value).map_err(|err| Error::cc(format!("Cannot stringify yaml value: {value:?}"), err))
}

pub fn stringify_multi(values: &[Value]) -> Result<String> {
	let mut out = String::new();
	for (i, val) in values.iter().enumerate() {
		if i > 0 {
			out.push_str("---\n");
		}
		let s = serde_yaml_ng::to_string(val).map_err(|err| Error::cc(format!("Cannot stringify multi-doc yaml at index {i}"), err))?;
		out.push_str(&s);
	}
	Ok(out)
}
