use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallData {
	pub pack_ref: String,
	pub run_args: Option<serde_json::Value>,
	pub needs_user_confirm: bool,
}
