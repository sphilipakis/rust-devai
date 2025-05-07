use crate::Result;
use crate::dir_context::DirContext;
use crate::exec::ExecutorSender;
use crate::run::{Literals, new_genai_client};
use crate::runtime::runtime_inner::RuntimeInner;
use crate::script::LuaEngine;
use genai::Client;
use std::sync::Arc;
use uuid::Uuid;

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
		let client = new_genai_client()?;

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

	pub fn session_str(&self) -> &str {
		self.inner.session().as_str()
	}

	pub fn session_uuid(&self) -> Uuid {
		self.inner.session().uuid()
	}
}

// region:    --- Session

#[derive(Debug, Clone)]
pub struct Session {
	uuid: Uuid,
	cached_str: Arc<str>,
}

impl Session {
	pub(super) fn new() -> Self {
		let uuid = Uuid::now_v7();
		let cached_str = Arc::from(uuid.to_string().as_str());
		Self { uuid, cached_str }
	}

	pub fn uuid(&self) -> Uuid {
		self.uuid
	}

	pub fn as_str(&self) -> &str {
		&self.cached_str
	}
}

// endregion: --- Session

// region:    --- Tests Support
#[cfg(test)]
mod tests_support {
	use super::*;
	use crate::_test_support::{SANDBOX_01_BASE_AIPACK_DIR, SANDBOX_01_WKS_DIR, gen_test_dir_path};
	use crate::dir_context::{AipackBaseDir, AipackPaths};
	use crate::exec::Executor;
	use crate::hub::{HubEvent, get_hub};
	use simple_fs::{SPath, ensure_dir};

	impl Runtime {
		/// This will create a new Runtime for the .tests-data/sandbox-01/ folder
		pub fn new_test_runtime_sandbox_01() -> Result<Self> {
			let current_dir = SPath::new(SANDBOX_01_WKS_DIR).canonicalize()?;
			let current_dir = SPath::new(current_dir);

			let wks_dir = current_dir.clone();

			let aipack_base_dir = SPath::new(SANDBOX_01_BASE_AIPACK_DIR).canonicalize()?;
			let aipack_base_dir = AipackBaseDir::new_for_test(aipack_base_dir)?;

			let aipack_paths = AipackPaths::from_aipack_base_and_wks_dirs(aipack_base_dir, wks_dir)?;

			let dir_context = DirContext::from_current_and_aipack_paths(current_dir, aipack_paths)?;

			Self::new_test_runtime(dir_context)
		}

		/// This dir is relative to `./tests-data/.tmp`
		pub fn new_test_runtime_for_temp_dir() -> Result<Self> {
			let test_dir = gen_test_dir_path();
			// should not be the case with the above case, but just as a double check
			if test_dir.path().is_absolute() {
				return Err(format!("temp dir cannot be absolute. Was '{test_dir}' ").into());
			}
			ensure_dir(&test_dir)?;
			let current_dir = test_dir.canonicalize()?;

			let wks_dir = current_dir.clone();

			// TODO: Probably do an base init for tose guys.

			ensure_dir(wks_dir.path())?;
			let base_aipack_dir = AipackBaseDir::new_for_test(current_dir.join(".aipack-base"))?;
			ensure_dir(&*base_aipack_dir)?;

			let aipack_paths = AipackPaths::from_aipack_base_and_wks_dirs(base_aipack_dir, wks_dir)?;

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
