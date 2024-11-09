use crate::hub::get_hub;
use crate::run::{get_genai_client, DirContext, RuntimeContext};
use crate::script::new_rhai_engine;
use crate::{Error, Result};
use flume::{Receiver, Sender};
use genai::Client;
use rhai::Engine;
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Clone)]
pub struct Runtime {
	context: RuntimeContext,
	rhai_engine: Arc<Engine>,
	stop_signal: Arc<watch::Sender<()>>, // Signal to stop the task
}

/// Constructors
impl Runtime {
	pub fn new(dir_context: DirContext) -> Result<Self> {
		// Note: Make the type explicit for clarity
		let (tx, rx): (Sender<Sender<Runtime>>, Receiver<Sender<Runtime>>) = flume::unbounded();
		let client = get_genai_client()?;

		let context = RuntimeContext::new(dir_context, client, tx);

		let rhai_engine = new_rhai_engine(context.clone())?;
		let rhai_engine = Arc::new(rhai_engine);

		let rx = Arc::new(rx);
		let (stop_signal, stop_receiver) = watch::channel(()); // Stop signal
		let stop_signal = Arc::new(stop_signal);
		let runtime = Self {
			context,
			rhai_engine,
			stop_signal,
		};

		// -- Process to listen for Runtime requests
		// NOTE: This is a workaround since we need the Runtime to have a rhai_engine,
		//       but we need the rhai_engine to be built with the RuntimeContext.
		//       For devai::run, the function will need to get the engine back.
		let runtime_for_rx = runtime.clone();
		tokio::spawn(async move {
			let mut stop_receiver = stop_receiver; // Keep the mutable receiver
			loop {
				tokio::select! {
					_ = stop_receiver.changed() => {
						// Stop signal received, exit loop
						break;
					}
					recv_result = rx.recv_async() => {
						match recv_result {
							Ok(one_tx) => {
								// Send back a clone of the runtime
								if let Err(send_err) = one_tx.send(runtime_for_rx.clone()) {
									get_hub().publish_sync(Error::cc("Runtime send error", send_err));
								}
							}
							Err(recv_err) => {
								get_hub().publish_sync(Error::cc("Runtime rx error", recv_err));
								break; // Exit loop on receiver error
							}
						}
					}
				}
			}
		});

		Ok(runtime)
	}

	#[cfg(test)]
	pub fn new_test_runtime_sandbox_01() -> Result<Self> {
		use crate::_test_support::SANDBOX_01_DIR;
		use simple_fs::SPath;

		let dir_context =
			DirContext::from_parent_dir_and_current_dir_for_test(SANDBOX_01_DIR, SPath::new(SANDBOX_01_DIR)?)?;
		Self::new(dir_context)
	}
}

/// We implement Drop to ensure we send an event to stop the process/task
/// that started in the new instance
impl Drop for Runtime {
	fn drop(&mut self) {
		// Notify the spawned task to stop
		let _ = self.stop_signal.send(());
	}
}

/// Getters
impl Runtime {
	#[allow(unused)]
	pub fn context(&self) -> RuntimeContext {
		self.context.clone()
	}

	pub fn rhai_engine(&self) -> &Arc<Engine> {
		&self.rhai_engine
	}

	pub fn genai_client(&self) -> &Client {
		self.context.genai_client()
	}

	pub fn dir_context(&self) -> &DirContext {
		self.context.dir_context()
	}
}