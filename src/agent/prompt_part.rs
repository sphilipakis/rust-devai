use crate::{Error, Result};
use genai::chat::ChatRole;
use lazy_regex::regex_captures;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PromptPart {
	pub kind: PartKind,
	pub content: String,
	pub options_str: Option<String>,
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

pub fn get_prompt_part_options_str(header: &str) -> Result<Option<String>> {
	if let Some((_, toml_part)) = regex_captures!(r"`([^`]+)`", header) {
		Ok(Some(toml_part.to_string()))
	} else {
		Ok(None)
	}
}

/// Parse options from a header string.
/// - toml format
/// - assume single line.
/// - Will add the `{ }` if not present
pub fn parse_prompt_part_options(content: &str) -> Result<Option<PartOptions>> {
	if content.trim().is_empty() {
		return Ok(None);
	}
	// Check if there's a backtick in the header
	// Add surrounding curly braces if not present
	let options_str = if content.starts_with('{') && content.ends_with('}') {
		content.to_string()
	} else {
		format!("{{{content}}}")
	};

	let options_str = format!("value = {options_str}");

	// Parse the TOML string into PartOptions
	let mut root: toml::Value = toml::from_str(&options_str).map_err(|_| {
		Error::custom(format!(
			r#"Prompt header options `{content}` format is invalid.
If set, it can only be `cache = true` or `cache = false`.
For example '# User `cache = true`'
If you used handlebars for some dynamic value `cache = data.should_cache_context`,
   make sure you use 'data.' and that the value `should_cache_context` is returned from your '# Data' Lua section.
"#
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
			"Prompt header '{content}' invalid format format is invalid. Cause: {err}"
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
		let options = parse_prompt_part_options("")?;
		assert!(options.is_none(), "should hava no options");

		// Test with options
		let options = parse_prompt_part_options("cache = true")?;
		let options = options.ok_or("Should have options")?;
		assert!(options.cache);

		// Test with spaces
		let options = parse_prompt_part_options("cache = false")?.ok_or("Should have options")?;
		assert!(!options.cache);

		// Test with already wrapped in braces
		let options = parse_prompt_part_options("{cache = true}")?.ok_or("Should have options")?;
		assert!(options.cache);

		// Test with future extensions
		let options = parse_prompt_part_options("cache = true, concurrency = 123")?.ok_or("Should have options")?;
		assert!(options.cache);
		// When we add concurrency field to PartOptions, we could check it here

		Ok(())
	}
}

// endregion: --- Tests
