use crate::Result;
use crate::event::{OneShotRx, OneShotTx, Tx, new_one_shot_channel};
use crate::run::{RunSubAgentParams, RunTopAgentParams};
use derive_more::From;

#[derive(Debug)]
pub struct RunQueueMessage {
	pub done_tx: RunDoneTx,
	pub action: RunQueueAction,
}

impl RunQueueMessage {
	pub fn new_and_rx(event: impl Into<RunQueueAction>) -> (Self, OneShotRx<()>) {
		let (tx, rx) = new_one_shot_channel::<()>("run_one_shot");
		let evt = Self {
			done_tx: tx.into(),
			action: event.into(),
		};
		(evt, rx)
	}
}

#[derive(Debug, From)]
pub enum RunQueueAction {
	RunTopAgent(RunTopAgentParams),
	RunSubAgent(RunSubAgentParams),
}

// region:    --- QueueTrx

#[derive(Debug, Clone, From)]
pub struct RunDoneTx(OneShotTx<()>);

impl RunDoneTx {
	pub async fn send(self, t: ()) -> Result<()> {
		self.0.send(()).await
	}
}

// endregion: --- QueueTrx
