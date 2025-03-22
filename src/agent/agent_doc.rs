use crate::Result;
use crate::agent::agent_options::AgentOptions;
use crate::agent::agent_ref::AgentRef;
use crate::agent::{Agent, AgentInner, PartKind, PromptPart, get_prompt_part_kind, get_prompt_part_options_str};
use crate::support::md::InBlockState;
use crate::support::tomls::parse_toml;
use genai::ModelName;
use simple_fs::{SPath, read_to_string};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub struct AgentDoc {
	spath: SPath,
	raw_content: String,
}

// region:    --- Capture State

#[derive(Debug)]
enum CaptureMode {
	None,

	// Below the output heading (perhaps not in a code block)
	BeforeAllSection,
	// Inside the code block
	BeforeAllCodeBlock,

	// Below the # Options section
	OptionsSection,
	OptionsTomlBlock,

	// Below the data heading (perhaps not in a code block)
	DataSection,
	// Inside the code block
	DataCodeBlock,

	PromptPart,

	// Below the output heading (perhaps not in a code block)
	OutputSection,
	// Inside the code block
	OutputCodeBlock,

	// Below the output heading (perhaps not in a code block)
	AfterAllSection,
	// Inside the code block
	AfterAllCodeBlock,
}

impl CaptureMode {
	/// Here we are inside a code block that is getting captured for an action
	/// either Lua script, toml, ...
	///
	/// NOTE: This is not used anymore since we have the `is_in_any_block`, but can be later.
	#[allow(unused)]
	fn is_inside_actionable_block(&self) -> bool {
		matches!(
			self,
			CaptureMode::OptionsTomlBlock
				| CaptureMode::BeforeAllCodeBlock
				| CaptureMode::DataCodeBlock
				| CaptureMode::OutputCodeBlock
				| CaptureMode::AfterAllCodeBlock
		)
	}

	/// Check if in a prompt part mode
	fn is_prompt_part(&self) -> bool {
		matches!(self, CaptureMode::PromptPart)
	}
}

// endregion: --- Capture State

/// Constructor
impl AgentDoc {
	pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
		let spath = SPath::from_std_path(path.as_ref())?;
		let raw_content = read_to_string(path)?;
		Ok(Self { spath, raw_content })
	}

	pub fn into_agent(self, name: &str, agent_ref: AgentRef, options: AgentOptions) -> Result<Agent> {
		let agent_inner = self.into_agent_inner(name, agent_ref, options)?;
		let agent = Agent::new(agent_inner)?;
		Ok(agent)
	}

	/// Internal method to create the first part of the agent inner
	/// This is sort of a Lexer, but very customize to extracting the Agent parts
	fn into_agent_inner(self, name: &str, agent_ref: AgentRef, agent_options: AgentOptions) -> Result<AgentInner> {
		let mut capture_mode = CaptureMode::None;

		// -- The buffers
		let mut options_toml: Vec<&str> = Vec::new();
		let mut before_all_script: Vec<&str> = Vec::new();
		let mut data_script: Vec<&str> = Vec::new();
		let mut output_script: Vec<&str> = Vec::new();
		let mut after_all_script: Vec<&str> = Vec::new();

		let mut prompt_parts: Vec<PromptPart> = Vec::new();
		// the vec String allow to be more efficient (as join later is more efficient)
		let mut current_part: Option<CurrentPromptPart> = None;

		// -- The actual parsing
		// NOTE: Need custom parser/lexer given the nature of the agent format.
		//       Markdown parsers tend to be lossless and would need wuite a bit of extra post-processing anyway.
		//       So, here we do one path, and capture what we need, exactly the way we need it

		let mut block_state = InBlockState::Out;
		let mut current_backticks = 0; // Track current block's backtick count (3 or 4)

		for line in self.raw_content.lines() {
			// Only update block state and check for backticks when not in prompt part mode
			if !capture_mode.is_prompt_part() {
				block_state = block_state.compute_new(line);
			}

			// If heading we decide the capture mode
			if (capture_mode.is_prompt_part() || block_state.is_out())
				&& line.starts_with('#')
				&& !line.starts_with("##")
			{
				let header_lower = line[1..].trim().to_lowercase();
				if header_lower == "options" {
					capture_mode = CaptureMode::OptionsSection;
				} else if header_lower == "before all" {
					capture_mode = CaptureMode::BeforeAllSection;
				} else if header_lower == "data" {
					capture_mode = CaptureMode::DataSection;
				} else if header_lower == "output" {
					capture_mode = CaptureMode::OutputSection;
				} else if header_lower == "after all" {
					capture_mode = CaptureMode::AfterAllSection;
				} else if let Some(part_kind) = get_prompt_part_kind(&header_lower) {
					capture_mode = CaptureMode::PromptPart;
					// TODO: will need to pass full case header in case we take string values in part options
					let part_options_str = get_prompt_part_options_str(&header_lower)?;
					// we finalize the previous part if present
					finalize_current_prompt_part(&mut current_part, &mut prompt_parts);
					// then, we create the new current_part
					current_part = Some(CurrentPromptPart(part_kind, part_options_str, Vec::new()));
				} else {
					// Stop processing current section if new top-level header
					capture_mode = CaptureMode::None;
				}
				continue;
			}

			// Only process code block markers for non-prompt part sections
			if !capture_mode.is_prompt_part() {
				// Check for code block markers with either 3 or 4 backticks
				let is_toml_block_start = line.starts_with("```toml") || line.starts_with("````toml");
				let is_lua_block_start = line.starts_with("```lua") || line.starts_with("````lua");
				let is_block_end = (line == "```" || line == "````")
					&& (current_backticks == 3 && line.starts_with("```")
						|| current_backticks == 4 && line.starts_with("````"));

				// Track the number of backticks used for the current block
				if is_toml_block_start || is_lua_block_start {
					current_backticks = if line.starts_with("````") { 4 } else { 3 };
				} else if is_block_end {
					current_backticks = 0;
				}

				match capture_mode {
					CaptureMode::None => {}

					// -- Options
					CaptureMode::OptionsSection => {
						if is_toml_block_start {
							capture_mode = CaptureMode::OptionsTomlBlock;
							continue;
						}
					}

					CaptureMode::OptionsTomlBlock => {
						if is_block_end {
							capture_mode = CaptureMode::None;
							continue;
						} else {
							push_line(&mut options_toml, line);
						}
					}

					// -- Before All
					CaptureMode::BeforeAllSection => {
						if is_lua_block_start {
							capture_mode = CaptureMode::BeforeAllCodeBlock;
							continue;
						}
					}
					CaptureMode::BeforeAllCodeBlock => {
						if is_block_end {
							capture_mode = CaptureMode::None;
							continue;
						} else {
							push_line(&mut before_all_script, line);
						}
					}

					// -- Data
					CaptureMode::DataSection => {
						if is_lua_block_start {
							capture_mode = CaptureMode::DataCodeBlock;
							continue;
						}
					}
					CaptureMode::DataCodeBlock => {
						if is_block_end {
							capture_mode = CaptureMode::None;
							continue;
						} else {
							push_line(&mut data_script, line);
						}
					}

					// -- Output
					CaptureMode::OutputSection => {
						if is_lua_block_start {
							capture_mode = CaptureMode::OutputCodeBlock;
							continue;
						}
					}
					CaptureMode::OutputCodeBlock => {
						if is_block_end {
							capture_mode = CaptureMode::None;
							continue;
						} else {
							push_line(&mut output_script, line);
						}
					}

					// -- After All
					CaptureMode::AfterAllSection => {
						if is_lua_block_start {
							capture_mode = CaptureMode::AfterAllCodeBlock;
							continue;
						}
					}
					CaptureMode::AfterAllCodeBlock => {
						if is_block_end {
							capture_mode = CaptureMode::None;
							continue;
						} else {
							push_line(&mut after_all_script, line);
						}
					}

					// For PromptPart, we handle it separately below
					CaptureMode::PromptPart => {}
				}
			} else {
				// For PromptPart, simply collect all lines as content
				if let Some(current_part) = &mut current_part {
					current_part.2.push(line);
				}
			}
		}

		// -- We finilize the last part if it was not closed
		finalize_current_prompt_part(&mut current_part, &mut prompt_parts);

		// -- Returning the data

		let options_toml = buffer_to_string(options_toml);

		let agent_options_ov: Option<AgentOptions> = if let Some(options_toml) = options_toml {
			Some(AgentOptions::from_options_value(parse_toml(&options_toml)?)?)
		} else {
			None
		};

		let agent_options = match agent_options_ov {
			Some(agent_options_ov) => agent_options.merge(agent_options_ov)?,
			None => agent_options,
		};

		// -- Get the model name
		let model_name = agent_options.model().map(ModelName::from);

		// -- Build the AgentInner
		let agent_inner = AgentInner {
			agent_options: Arc::new(agent_options),

			name: name.to_string(),
			agent_ref,

			file_name: self.spath.name().to_string(),
			file_path: self.spath.as_str().to_string(),

			model_name,

			before_all_script: buffer_to_string(before_all_script),
			data_script: buffer_to_string(data_script),

			prompt_parts,

			output_script: buffer_to_string(output_script),
			after_all_script: buffer_to_string(after_all_script),
		};

		Ok(agent_inner)
	}
}

/// Constructor for test
#[cfg(test)]
impl AgentDoc {
	pub fn from_content(spath: impl AsRef<Path>, content: impl Into<String>) -> Result<Self> {
		let spath = SPath::from_std_path(spath.as_ref())?;
		let raw_content = content.into();
		Ok(Self { spath, raw_content })
	}
}

// region:    --- Support

/// Type of the function below and the `into_agent_inner` lexer
/// (PartKind, PartOptionsStr, Content)
struct CurrentPromptPart<'a>(PartKind, Option<String>, Vec<&'a str>);

/// Finalize a eventual current_part
fn finalize_current_prompt_part(current_part: &mut Option<CurrentPromptPart<'_>>, prompt_parts: &mut Vec<PromptPart>) {
	if let Some(current_part) = current_part.take() {
		// to have the last line
		let kind = current_part.0;
		let options_str = current_part.1;
		let mut content = current_part.2;
		content.push("");
		let content = content.join("\n");

		let part = PromptPart {
			kind,
			options_str,
			content,
		};
		prompt_parts.push(part);
	}
}

/// Push a new line and the a \n to respect the new line
fn push_line<'a, 'b, 'c: 'b>(content: &'a mut Vec<&'b str>, line: &'c str) {
	content.push(line);
	content.push("\n");
}

fn buffer_to_string(content: Vec<&str>) -> Option<String> {
	if content.is_empty() {
		None
	} else {
		Some(content.join(""))
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::assert_contains;
	use serde_json::json;

	#[tokio::test]
	async fn test_agent_doc_not_so_good_parse() -> Result<()> {
		// -- Setup & Fixtures
		let path = "tests-data/sandbox-01/agent-script/agent-not-so-good.aip";
		let agent_doc = AgentDoc::from_file(path)?;
		let agent_ref = AgentRef::LocalPath("tests-data/sandbox-01/agent-script/agent-not-so-good.aip".into());

		// -- Exec
		let agent = agent_doc.into_agent(
			"mock-agent",
			agent_ref,
			AgentOptions::from_options_value(json!({"model": "mockmodel"}))?,
		)?;

		// -- Check
		assert!(agent.output_script().is_some(), "Output script should be present");
		assert_eq!(agent.output_script().unwrap().trim(), "return data.one");

		Ok(())
	}

	#[tokio::test]
	async fn test_agent_doc_multi_system() -> Result<()> {
		// -- Setup & Fixtures
		let content = r#"
# System

System 1

# System 
System 2

# User

User 1

# Output

```lua
return "hello"
```
		"#;
		let agent_doc = AgentDoc::from_content("mock-agent.aip", content)?;
		let agent_ref = AgentRef::LocalPath("mock-agent.aip".into());

		// -- Exec
		let agent = agent_doc.into_agent(
			"mock-agent",
			agent_ref,
			AgentOptions::from_options_value(json!({"model": "mock-model"}))?,
		)?;

		// -- Check
		let parts = agent.prompt_parts();
		assert_eq!(parts.len(), 3);
		assert_contains(&parts[0].content, "System 1");
		assert_contains(&parts[1].content, "System 2");
		assert_contains(&parts[2].content, "User 1");

		Ok(())
	}
}

// endregion: --- Tests
