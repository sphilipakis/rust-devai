use crate::model::ScalarEnum;
use crate::support::text::truncate;
use macro_rules_attribute as mra;
use serde_json::Value;
use tracing::error;
use uuid::Uuid;
use value_ext::JsonValueExt;

const SHORT_MAX_CHAR_LENGTH: usize = 64;

#[derive(Debug, Clone)]
pub struct TypedContent {
	pub uid: Uuid,
	pub typ: ContentTyp,
	pub content: Option<String>,
	pub display: Option<String>,
}

#[mra::derive(Debug, ScalarEnum!)]
pub enum ContentTyp {
	Json,
	Text,
}

impl TypedContent {
	/// Return the short content, and bool if there is more content or short is all of the content
	pub fn extract_short(&self) -> (Option<String>, bool) {
		match (self.display.as_ref(), self.content.as_ref()) {
			// no short at all and not more content
			(None, None) => (None, false),
			// twe
			(None, Some(content)) => {
				let (short, has_truncated) = truncate(content, SHORT_MAX_CHAR_LENGTH);
				(Some(short.to_string()), has_truncated)
			}
			(Some(display), _) => {
				let (short, has_truncated) = truncate(display, SHORT_MAX_CHAR_LENGTH);
				(Some(short.to_string()), has_truncated)
			}
		}
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
