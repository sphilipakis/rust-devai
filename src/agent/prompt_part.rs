use crate::{Error, Result};
use genai::chat::ChatRole;
use lazy_regex::regex_captures;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PromptPart {
	pub kind: PartKind,
	pub content: String,
	pub options: Option<PartOptions>,
}

#[derive(Debug, Clone)]
pub enum PartKind {
	Instruction,
	System,
	Assistant,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartOptions {
	pub cache: bool,
}

// region:    --- Froms

impl From<PartKind> for ChatRole {
	fn from(kind: PartKind) -> Self {
		match kind {
			PartKind::Instruction => ChatRole::User,
			PartKind::System => ChatRole::System,
			PartKind::Assistant => ChatRole::Assistant,
		}
	}
}

impl From<&PartKind> for ChatRole {
	fn from(kind: &PartKind) -> Self {
		match kind {
			PartKind::Instruction => ChatRole::User,
			PartKind::System => ChatRole::System,
			PartKind::Assistant => ChatRole::Assistant,
		}
	}
}

// endregion: --- Froms

// region:    --- Parsers

/// Extract and parse options from a header string.
///
/// Examples:
/// - "user `cache = true`" -> PartOptions { cache: true }
/// - "system  " -> PartOptions::default()
/// - "user" -> PartOptions::default()
pub fn get_prompt_part_options(header: &str) -> Result<Option<PartOptions>> {
	// Check if there's a backtick in the header
	let options_str = if let Some((_, toml_part)) = regex_captures!(r"`([^`]+)`", header) {
		// Extract the content between backticks
		let toml_content = toml_part.trim();

		// Add surrounding curly braces if not present
		let toml_content = if toml_content.starts_with('{') && toml_content.ends_with('}') {
			toml_content.to_string()
		} else {
			format!("{{{}}}", toml_content)
		};

		format!("value = {toml_content}")
	} else {
		// No backtick found, return default options
		return Ok(None);
	};

	// Parse the TOML string into PartOptions
	let mut root: toml::Value = toml::from_str(&options_str).map_err(|err| {
		Error::custom(format!(
			"Prompt header '{header}' invalid format format is invalid. Cause: {err}"
		))
	})?;

	// Ensure `root` is a table and extract the value from it
	let value = if let toml::Value::Table(root) = &mut root {
		root.remove("value")
	} else {
		None
	};

	let Some(value) = value else {
		return Ok(None);
	};

	let options: PartOptions = value.try_into().map_err(|err| {
		Error::custom(format!(
			"Prompt header '{header}' invalid format format is invalid. Cause: {err}"
		))
	})?;

	Ok(Some(options))
}

/// This assume header is lowercase
pub fn get_prompt_part_kind(header: &str) -> Option<PartKind> {
	let header = header.split_once('`').map(|(before, _)| before).unwrap_or(header);
	let header = header.trim();
	if header == "user" || header == "inst" || header == "instruction" {
		Some(PartKind::Instruction)
	} else if header == "system" {
		Some(PartKind::System)
	} else if header == "assistant" || header == "model" || header == "mind trick" || header == "jedi trick" {
		Some(PartKind::Assistant)
	} else {
		None
	}
}

// endregion: --- Parsers

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_agent_prompt_part_parse_options() -> Result<()> {
		// Test with no options
		let options = get_prompt_part_options("user")?;
		assert!(options.is_none(), "should hava no options");

		// Test with options
		let options = get_prompt_part_options("user `cache = true`")?;
		let options = options.ok_or("Should have options")?;
		assert!(options.cache);

		// Test with spaces
		let options = get_prompt_part_options("system  `cache = false`")?.ok_or("Should have options")?;
		assert!(!options.cache);

		// Test with already wrapped in braces
		let options = get_prompt_part_options("user `{cache = true}`")?.ok_or("Should have options")?;
		assert!(options.cache);

		// Test with future extensions
		let options =
			get_prompt_part_options("user `cache = true, concurrency = 123`")?.ok_or("Should have options")?;
		assert!(options.cache);
		// When we add concurrency field to PartOptions, we could check it here

		Ok(())
	}
}

// endregion: --- Tests
