use crate::dir_context::DirContext;
use genai::Client;

#[derive(Clone)]
pub struct RuntimeInner {
	dir_context: DirContext,
	genai_client: Client,
}

/// Constructors
impl RuntimeInner {
	pub fn new(dir_context: DirContext, genai_client: Client) -> Self {
		Self {
			dir_context,
			genai_client,
		}
	}
}

/// Getters
impl RuntimeInner {
	pub fn dir_context(&self) -> &DirContext {
		&self.dir_context
	}

	pub fn genai_client(&self) -> &Client {
		&self.genai_client
	}
}
