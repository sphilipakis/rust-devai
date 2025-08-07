use crate::Error;
use crate::hub::get_hub;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// List of common API key environment variables to check.
pub const KEY_ENV_VARS: &[&str] = &[
	"OPENAI_API_KEY",
	"ANTHROPIC_API_KEY",
	"GEMINI_API_KEY",
	"FIREWORKS_API_KEY",
	"TOGETHER_API_KEY",
	"NEBIUS_API_KEY",
	"XAI_API_KEY",
	"DEEPSEEK_API_KEY",
	"GROQ_API_KEY",
	"COHERE_API_KEY",
];

/// Checks the environment for a predefined list of API keys
/// and returns a set containing the names of the keys that are set and non-empty.
pub fn get_available_api_keys() -> HashSet<String> {
	let mut available_keys = HashSet::new();
	for &key in KEY_ENV_VARS {
		match std::env::var(key) {
			Ok(val) if !val.trim().is_empty() => {
				available_keys.insert(key.to_string());
			}
			_ => (), // Key not set or empty
		}
	}
	available_keys
}

/// Attempt to open a path via vscode
/// NOTE: VSCode will do the right thing when the user have multiple vscode open
///       by opening the path in the corresponding workspace.
pub async fn open_vscode(path: impl AsRef<Path>) {
	let path = path.as_ref();

	let output = if cfg!(target_os = "windows") {
		Command::new("cmd")
			// for path.to_str().unwrap..., should never happen, but should never crash either
			.args(["/C", "code", path.to_str().unwrap_or_default()])
			.output()
	} else {
		Command::new("code").arg(path).output()
	};

	match output {
		Ok(output) if output.status.success() => {}
		Ok(output) => {
			let msg = format!(
				"Error opening VSCode:\nstdout: {}\nstderr: {}",
				String::from_utf8_lossy(&output.stdout),
				String::from_utf8_lossy(&output.stderr)
			);
			get_hub().publish(Error::Custom(msg)).await;
		}
		Err(e) => {
			let msg = format!("Failed to execute VSCode command: {e}");
			get_hub().publish(Error::Custom(msg)).await;
		}
	}
}
