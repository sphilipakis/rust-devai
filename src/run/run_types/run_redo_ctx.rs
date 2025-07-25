use crate::agent::Agent;
use crate::run::RunTopAgentParams;
use crate::runtime::Runtime;
use std::sync::Arc;

// #[derive(From)]
// pub enum RedoCtx {
// 	RunRedoCtx(Arc<RunRedoCtx>),
// }

// impl From<RunRedoCtx> for RedoCtx {
// 	fn from(run_redo_ctx: RunRedoCtx) -> Self {
// 		RedoCtx::RunRedoCtx(run_redo_ctx.into())
// 	}
// }

// impl RedoCtx {
// 	pub fn get_agent(&self) -> Option<&Agent> {
// 		match self {
// 			RedoCtx::RunRedoCtx(redo_ctx) => Some(redo_ctx.agent()),
// 		}
// 	}
// }

#[derive(Debug, Clone)]
pub struct RunRedoCtx {
	inner: Arc<CtxInner>,
}

/// constructor
impl RunRedoCtx {
	pub fn new(runtime: Runtime, agent: Agent, run_options: RunTopAgentParams) -> Self {
		Self {
			inner: Arc::new(CtxInner {
				runtime,
				agent,
				run_options,
			}),
		}
	}
}

/// getters
impl RunRedoCtx {
	pub fn runtime(&self) -> &Runtime {
		&self.inner.runtime
	}

	pub fn agent(&self) -> &Agent {
		&self.inner.agent
	}

	pub fn run_options(&self) -> &RunTopAgentParams {
		&self.inner.run_options
	}
}

/// A Context that hold the information to redo this run
#[derive(Debug)]
struct CtxInner {
	runtime: Runtime,
	agent: Agent,
	run_options: RunTopAgentParams,
}
