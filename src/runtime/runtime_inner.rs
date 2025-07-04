use crate::dir_context::DirContext;
use crate::exec::ExecutorSender;
use crate::runtime::Session;
use genai::Client;

#[derive(Debug, Clone)]
pub struct RuntimeInner {
	pub(super) dir_context: DirContext,
	pub(super) genai_client: Client,
	pub(super) executor_sender: ExecutorSender,
	pub(super) session: Session,
}

/// Constructors
impl RuntimeInner {
	// pub fn new(dir_context: DirContext, genai_client: Client, executor_sender: ExecutorSender) -> Self {
	// 	Self {
	// 		dir_context,
	// 		genai_client,
	// 		executor_sender,
	// 		session: ,
	// 	}
	// }
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

	pub fn session(&self) -> &Session {
		&self.session
	}
}
