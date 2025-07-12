use crate::derive_simple_enum_type;
use serde_json::Value;
use tracing::error;
use uuid::Uuid;
use value_ext::JsonValueExt;

#[derive(Debug, Clone)]
pub struct TypedContent {
	pub uid: Uuid,
	pub typ: ContentTyp,
	pub content: String,
}

derive_simple_enum_type! {
pub enum ContentTyp {
	Json,
	Text,
}
}

impl TypedContent {
	/// New from a new Value (will create new UUID)
	pub fn from_value(value: &Value) -> Option<Self> {
		// NOTE: since Value::Null will be None, it will usually not be updated a null in the db
		//       Might need to fix that eventually
		match value {
			Value::Null => None,
			Value::String(content) => Some(Self {
				uid: Uuid::now_v7(),
				typ: ContentTyp::Text,
				content: content.to_string(),
			}),
			other => {
				//
				match other.x_pretty() {
					Ok(content) => Some(Self {
						uid: Uuid::now_v7(),
						typ: ContentTyp::Json,
						content,
					}),
					Err(err) => {
						error!("Error stringify input: {err}");
						Some(Self {
							uid: Uuid::now_v7(),
							typ: ContentTyp::Json,
							content: format!("Error stringify input: {err}"),
						})
					}
				}
			}
		}
	}
}
