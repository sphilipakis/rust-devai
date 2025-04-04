use crate::Result;
use crate::dir_context::DirContext;
use crate::exec::ExecutorSender;
use crate::run::{Literals, get_genai_client};
use crate::runtime::runtime_inner::RuntimeInner;
use crate::script::LuaEngine;
use genai::Client;
use std::sync::Arc;

/// The runtime that holds the RuntimeInner (under Arc) with the genai client and dir_context
/// This type is made to be cloned when passed to different part of the system.
///
/// Note: Should not create new Runtimes except for test. Just clone the one available.
#[derive(Debug, Clone)]
pub struct Runtime {
	inner: Arc<RuntimeInner>,
}

/// Constructors
impl Runtime {
	/// Create a new Runtime from a dir_context (.aipack and .aipack-base)
	/// This is called when the cli start a command
	pub fn new(dir_context: DirContext, exec_sender: ExecutorSender) -> Result<Self> {
		// Note: Make the type explicit for clarity
		let client = get_genai_client()?;

		let inner = RuntimeInner::new(dir_context, client, exec_sender);

		let runtime = Self { inner: Arc::new(inner) };

		Ok(runtime)
	}
}

/// lua engine
/// NOTE: For now, we do not keep any Lau engine in the Runtime, but just create new ones.
///       Later, we might have an optmized reuse strategy of lua engines (but need to be cautious as not multi-threaded)
impl Runtime {
	pub fn new_lua_engine_with_ctx(&self, ctx: &Literals) -> Result<LuaEngine> {
		LuaEngine::new_with_ctx(self.clone(), ctx)
	}

	#[cfg(test)]
	pub fn new_lua_engine_without_ctx_test_only(&self) -> Result<LuaEngine> {
		LuaEngine::new(self.clone())
	}
}

/// Getters
impl Runtime {
	pub fn genai_client(&self) -> &Client {
		self.inner.genai_client()
	}

	pub fn dir_context(&self) -> &DirContext {
		self.inner.dir_context()
	}

	pub fn executor_sender(&self) -> ExecutorSender {
		self.inner.executor_sender()
	}
}

// region:    --- Tests Support
#[cfg(test)]
mod tests_support {
	use super::*;
	use crate::_test_support::{SANDBOX_01_BASE_AIPACK_DIR, SANDBOX_01_WKS_DIR, gen_test_dir_path};
	use crate::dir_context::AipackPaths;
	use crate::exec::Executor;
	use crate::hub::{HubEvent, get_hub};
	use simple_fs::{SPath, ensure_dir};

	impl Runtime {
		/// This will create a new Runtime for the .tests-data/sandbox-01/ folder
		pub fn new_test_runtime_sandbox_01() -> Result<Self> {
			let current_dir = SPath::new(SANDBOX_01_WKS_DIR).canonicalize()?;
			let current_dir = SPath::new(current_dir);

			let wks_aipack_dir = current_dir.join(".aipack");

			let base_aipack_dir = SPath::new(SANDBOX_01_BASE_AIPACK_DIR).canonicalize()?;

			let aipack_paths = AipackPaths::from_aipack_base_and_wks_dirs(base_aipack_dir, wks_aipack_dir)?;

			let dir_context = DirContext::from_current_and_aipack_paths(current_dir, aipack_paths)?;

			Self::new_test_runtime(dir_context)
		}

		/// This dir is relative to `./tests-data/.tmp`
		pub fn new_test_runtime_for_temp_dir() -> Result<Self> {
			let current_dir = gen_test_dir_path();
			// should not be the case with the above case, but just as a double prec
			if current_dir.path().is_absolute() {
				return Err(format!("temp dir cannot be absolute. Was '{current_dir}' ").into());
			}
			ensure_dir(&current_dir)?;
			let current_dir = current_dir.canonicalize()?;

			// TODO: Probably do an base init for tose guys.
			let wks_aipack_dir = current_dir.join(".aipack");
			ensure_dir(&wks_aipack_dir)?;
			let base_aipack_dir = current_dir.join(".aipack-base");
			ensure_dir(&base_aipack_dir)?;

			let aipack_paths = AipackPaths::from_aipack_base_and_wks_dirs(base_aipack_dir, wks_aipack_dir)?;

			let dir_context = DirContext::from_current_and_aipack_paths(current_dir, aipack_paths)?;

			Self::new_test_runtime(dir_context)
		}

		fn new_test_runtime(dir_context: DirContext) -> Result<Self> {
			let executor = Executor::new();
			let exec_sender = executor.sender();
			tokio::spawn(async move {
				if let Err(err) = executor.start().await {
					let hub = get_hub();
					hub.publish(HubEvent::Error { error: err.into() }).await;
					hub.publish(HubEvent::Quit).await;
				}
			});
			Self::new(dir_context, exec_sender)
		}
	}
}

// endregion: --- Tests Support
