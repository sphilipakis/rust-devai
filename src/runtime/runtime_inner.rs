use crate::dir_context::DirContext;
use crate::exec::ExecutorSender;
use genai::Client;

#[derive(Clone)]
pub struct RuntimeInner {
	dir_context: DirContext,
	genai_client: Client,
	executor_sender: ExecutorSender,
}

/// Constructors
impl RuntimeInner {
	pub fn new(dir_context: DirContext, genai_client: Client, executor_sender: ExecutorSender) -> Self {
		Self {
			dir_context,
			genai_client,
			executor_sender,
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

	pub fn executor_sender(&self) -> ExecutorSender {
		self.executor_sender.clone()
	}
}
