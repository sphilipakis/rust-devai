use crate::event::{Rx, Tx, new_channel};
use crate::hub::get_hub;
use crate::run::run_executor::RunQueueMessage;
use crate::run::run_executor::run_queue_event::RunQueueAction;
use crate::runtime::Runtime;
use crate::{Error, Result};
use derive_more::{Deref, From};

#[derive(Debug, Clone, From, Deref)]
pub struct RunQueueTx(Tx<RunQueueMessage>);

pub struct RunQueueExecutor {
	rx: Rx<RunQueueMessage>,
	_tx: RunQueueTx,
}

impl RunQueueExecutor {
	pub fn new() -> Self {
		let (tx, rx) = new_channel::<RunQueueMessage>("run_queue_executor");
		Self { rx, _tx: tx.into() }
	}

	/// Consume the key, start it.
	/// NOTE: Make sure to keep at least one ref of the returned tx
	pub fn start(self) -> RunQueueTx {
		tokio::spawn(async move {
			let hub = get_hub();
			loop {
				let mut msg = match self.rx.recv().await {
					Ok(msg) => msg,
					Err(err) => {
						hub.publish(Error::cc("Fail in RunQueueExecutor recv", err));
						continue;
					}
				};

				let tx = msg.done_tx;
				tx.send(()).await;
			}
		});

		self._tx.clone()
	}
}
