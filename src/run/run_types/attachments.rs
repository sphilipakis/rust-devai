use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

// region:    --- Attachment

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
	// For now, only support file local path.
	pub file_source: String,
	pub file_name: Option<String>,
	pub title: Option<String>,
}

// endregion: --- Attachment

// region:    --- Attachments

/// A collection of attachments, supporting deserialization from a list of attachments, a single attachment object, or null.
#[derive(Debug, Clone, Serialize)]
pub struct Attachments {
	pub list: Vec<Attachment>,
}

impl Attachments {
	pub fn new(list: Vec<Attachment>) -> Self {
		Self { list }
	}
}

impl<'de> Deserialize<'de> for Attachments {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let value = Value::deserialize(deserializer)?;

		let list = match value {
			Value::Array(arr) => arr
				.into_iter()
				.map(serde_json::from_value)
				.collect::<Result<Vec<Attachment>, _>>()
				.map_err(|e| DeError::custom(format!("Failed to deserialize array elements into Attachment: {e}")))?,
			Value::Null => Vec::new(),
			// Allow single object to be deserialized as a single-item list
			Value::Object(obj) => {
				if obj.is_empty() {
					Vec::new()
				} else {
					let single_attachment: Attachment = serde_json::from_value(Value::Object(obj)).map_err(|e| {
						DeError::custom(format!(
							"Failed to parse 'attachments' because of wrong format.\nCause: {e}"
						))
					})?;

					vec![single_attachment]
				}
			}
			other => {
				return Err(DeError::custom(format!(
					"Expected an array of attachments, a single attachment object, or null, found: {}",
					other
				)));
			}
		};

		Ok(Attachments::new(list))
	}
}

impl IntoIterator for Attachments {
	type Item = Attachment;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.list.into_iter()
	}
}

impl<'a> IntoIterator for &'a Attachments {
	type Item = &'a Attachment;
	type IntoIter = std::slice::Iter<'a, Attachment>;

	fn into_iter(self) -> Self::IntoIter {
		self.list.iter()
	}
}

// endregion: --- Attachments
