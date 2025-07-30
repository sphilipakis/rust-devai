use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Marker {
	pub label: String,
	pub content: String,
}
