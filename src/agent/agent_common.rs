use crate::agent::PromptPart;
use crate::agent::agent_options::AgentOptions;
use crate::agent::agent_ref::AgentRef;
use crate::{Error, Result};
use genai::ModelName;
use genai::chat::ChatOptions;
use simple_fs::SPath;
use std::sync::Arc;

/// A sync efficient & friendly Agent containing the AgentInner
#[derive(Debug, Clone)]
pub struct Agent {
	inner: Arc<AgentInner>,
	model: ModelName,
	model_resolved: ModelName,
	agent_options_ov: Option<Arc<AgentOptions>>,
	genai_chat_options: Arc<ChatOptions>,
}

/// Constructor from AgentInner
///
/// TODO: Make it DRYer
impl Agent {
	pub(super) fn new(agent_inner: AgentInner) -> Result<Agent> {
		let inner = Arc::new(agent_inner);

		// -- Build the model and model_resolved
		let model = inner.model_name.clone().ok_or_else(|| Error::ModelMissing {
			agent_path: inner.file_path.to_string(),
		})?;
		let model_resolved = inner.agent_options.resolve_model().map(|v| v.into()).unwrap_or(model.clone());

		let chat_options = ChatOptions::from(&*inner.agent_options);

		Ok(Agent {
			inner,
			model,
			model_resolved,
			agent_options_ov: None,
			genai_chat_options: chat_options.into(),
		})
	}

	pub fn new_merge(&self, options: AgentOptions) -> Result<Agent> {
		let options = self.options().merge_new(options)?;
		let inner = self.inner.clone();

		// -- Build the model and model_resolved
		let model = options.model().map(ModelName::from).ok_or_else(|| Error::ModelMissing {
			agent_path: inner.file_path.to_string(),
		})?;
		let model_resolved = options.resolve_model().map(|v| v.into()).unwrap_or(model.clone());

		// -- Build the genai chat optoins
		let chat_options = ChatOptions::from(&options);

		// -- Returns
		Ok(Agent {
			inner,
			model,
			model_resolved,
			agent_options_ov: Some(Arc::new(options)),
			genai_chat_options: chat_options.into(),
		})
	}
}

/// Getters
impl Agent {
	pub fn model(&self) -> &ModelName {
		&self.model
	}

	pub fn model_resolved(&self) -> &ModelName {
		&self.model_resolved
	}

	pub fn genai_chat_options(&self) -> &ChatOptions {
		&self.genai_chat_options
	}

	pub fn options(&self) -> Arc<AgentOptions> {
		self.agent_options_ov
			.clone()
			.unwrap_or_else(|| self.inner.agent_options.clone())
	}

	pub fn options_as_ref(&self) -> &AgentOptions {
		self.agent_options_ov
			.as_ref()
			.map(|o| o.as_ref())
			.unwrap_or(&self.inner.agent_options)
	}

	pub fn agent_ref(&self) -> &AgentRef {
		&self.inner.agent_ref
	}

	pub fn name(&self) -> &str {
		&self.inner.name
	}

	#[allow(unused)]
	pub fn file_name(&self) -> &str {
		&self.inner.file_name
	}

	pub fn file_path(&self) -> &str {
		&self.inner.file_path
	}

	pub fn file_dir(&self) -> Result<SPath> {
		Ok(SPath::new(&self.inner.file_path)
			.parent()
			.ok_or("Agent does not have a parent dir")?)
	}

	pub fn before_all_script(&self) -> Option<&str> {
		self.inner.before_all_script.as_deref()
	}

	pub fn prompt_parts(&self) -> Vec<&PromptPart> {
		self.inner.prompt_parts.iter().collect()
	}

	pub fn data_script(&self) -> Option<&str> {
		self.inner.data_script.as_deref()
	}

	pub fn output_script(&self) -> Option<&str> {
		self.inner.output_script.as_deref()
	}

	pub fn after_all_script(&self) -> Option<&str> {
		self.inner.after_all_script.as_deref()
	}
}

/// Peekers
impl Agent {
	pub fn has_prompt_parts(&self) -> bool {
		!self.inner.prompt_parts.is_empty()
	}

	pub fn has_some_task_stages(&self) -> bool {
		!self.inner.prompt_parts.is_empty() || self.inner.data_script.is_some() || self.inner.output_script.is_some()
	}
}

// Some test implementations
#[cfg(test)]
mod for_test {
	use super::*;
	use crate::agent::AgentDoc;
	use serde_json::json;

	impl Agent {
		/// Create a mock agent from content only
		pub fn mock_from_content(content: &str) -> Result<Agent> {
			let agent_file_name = "mock-agent.aip";
			let agent_ref = AgentRef::LocalPath(agent_file_name.into());
			let doc = AgentDoc::from_content(agent_file_name, content)?;
			let agent = doc.into_agent(
				"mock-agent",
				agent_ref,
				AgentOptions::from_options_value(json!({"model": "mock-model"}))?,
			)?;
			Ok(agent)
		}

		/// Create a mod agent from file
		#[allow(unused)]
		pub fn mock_from_file(path: &str) -> Result<Agent> {
			let agent_doc = AgentDoc::from_file(path)?;
			let agent_ref = AgentRef::LocalPath(path.into());
			let spath = SPath::new(path);

			let agent = agent_doc.into_agent(
				spath.stem(),
				agent_ref,
				AgentOptions::from_options_value(json!({"model": "mock-model"}))?,
			)?;

			Ok(agent)
		}
	}
}

// region:    --- AgentInner

/// AgentInner is ok to be public to allow user-code to build Agent simply.
#[derive(Debug, Clone)]
pub(super) struct AgentInner {
	pub name: String,

	#[allow(unused)]
	pub agent_ref: AgentRef,

	pub file_name: String,
	pub file_path: String,

	pub agent_options: Arc<AgentOptions>,

	/// The model that came from the options
	pub model_name: Option<ModelName>,

	pub before_all_script: Option<String>,

	/// Contains the instruction, system, assistant in order of the file
	pub prompt_parts: Vec<PromptPart>,

	/// Script
	pub data_script: Option<String>,
	pub output_script: Option<String>,
	pub after_all_script: Option<String>,
}

// endregion: --- AgentInner

#[cfg(test)]
#[path = "../_tests/tests_agent_parse.rs"]
mod tests_agent_parse;
