use crate::derive_simple_enum_type;
use serde_json::Value;
use tracing::error;
use uuid::Uuid;
use value_ext::JsonValueExt;

#[derive(Debug, Clone)]
pub struct TypedContent {
	pub uid: Uuid,
	pub typ: ContentTyp,
	pub content: Option<String>,
	pub display: Option<String>,
}

derive_simple_enum_type! {
pub enum ContentTyp {
	Json,
	Text,
}
}

impl TypedContent {
	/// New from a new Value (will create new UUID)
	pub fn from_value(value: &Value) -> Self {
		// NOTE: since Value::Null will be None, it will usually not be updated a null in the db
		//       Might need to fix that eventually
		match value {
			Value::Null => Self {
				uid: Uuid::now_v7(),
				typ: ContentTyp::Text,
				content: None,
				display: None,
			},
			Value::String(content) => Self {
				uid: Uuid::now_v7(),
				typ: ContentTyp::Text,
				content: Some(content.to_string()),
				display: None,
			},
			other => {
				// -- extract the potential display
				let display = other.get("_display");
				let display = if let Some(display) = display {
					match display {
						Value::Null => None,
						Value::String(display) => Some(display.clone()),
						other_display => match other_display.x_pretty() {
							Ok(display) => Some(display),
							Err(err) => Some(format!("Cannot serialize input._display: {err}")),
						},
					}
				} else {
					None
				};
				//
				match other.x_pretty() {
					Ok(content) => Self {
						uid: Uuid::now_v7(),
						typ: ContentTyp::Json,
						content: Some(content),
						display,
					},
					Err(err) => {
						error!("Error stringify input: {err}");
						Self {
							uid: Uuid::now_v7(),
							typ: ContentTyp::Json,
							content: Some(format!("Error stringify input: {err}")),
							display,
						}
					}
				}
			}
		}
	}
}
