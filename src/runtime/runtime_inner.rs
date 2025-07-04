use crate::dir_context::DirContext;
use crate::exec::ExecutorTx;
use crate::runtime::Session;
use crate::runtime::queue::RunTx;
use crate::store::ModelManager;
use genai::Client;

#[derive(Debug, Clone)]
pub struct RuntimeInner {
	pub(super) dir_context: DirContext,
	pub(super) genai_client: Client,
	pub(super) executor_tx: ExecutorTx,
	pub(super) session: Session,
	pub(super) run_tx: RunTx,
	pub(super) mm: ModelManager,
}

/// Getters
impl RuntimeInner {
	pub fn dir_context(&self) -> &DirContext {
		&self.dir_context
	}

	pub fn genai_client(&self) -> &Client {
		&self.genai_client
	}

	pub fn executor_tx(&self) -> ExecutorTx {
		self.executor_tx.clone()
	}

	pub fn run_tx(&self) -> &RunTx {
		&self.run_tx
	}

	pub fn session(&self) -> &Session {
		&self.session
	}
}
