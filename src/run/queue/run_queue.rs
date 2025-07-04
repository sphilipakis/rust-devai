use super::run_event::RunEvent;
use crate::Result;
use flume::{Receiver, Sender};

// region:    --- RunQueue

/// A queue for `RunEvent`s with a single consumer (the queue `start` loop)
/// and multiple producers.
pub struct RunQueue {
	tx: RunTx,
	rx: Option<RunRx>,
}

impl RunQueue {
	/// Creates a new `RunQueue`.
	pub fn new() -> Self {
		let (tx, rx) = flume::unbounded();
		Self {
			tx: RunTx(tx),
			rx: Some(RunRx(rx)),
		}
	}

	/// Starts the queue consumer loop and returns a `RunTx` sender.
	/// This must be called once.
	pub fn start(&mut self) -> Result<RunTx> {
		let rx = self.rx.take().ok_or_else(|| crate::Error::custom("RunQueue already started"))?;

		tokio::spawn(async move {
			while let Ok(event) = rx.recv().await {
				// For now, just print the event, will be replaced by a proper handler.
				println!("[RunEvent] - {event:?}");
			}
		});

		Ok(self.sender())
	}

	/// Returns a `RunTx` sender to send events to this queue.
	pub fn sender(&self) -> RunTx {
		self.tx.clone()
	}
}

// endregion: --- RunQueue

// region:    --- RunTx/Rx

#[derive(Clone)]
pub struct RunTx(Sender<RunEvent>);

impl RunTx {
	/// Sends an event asynchronously.
	pub async fn send(&self, event: impl Into<RunEvent>) -> Result<()> {
		let event = event.into();
		self.0
			.send_async(event)
			.await
			.map_err(|_| crate::Error::custom("RunQueue send async error, receiver dropped before sender"))
	}

	/// Sends an event synchronously (blocking).
	pub fn send_sync(&self, event: impl Into<RunEvent>) -> Result<()> {
		let event = event.into();
		self.0
			.send(event)
			.map_err(|_| crate::Error::custom("RunQueue send sync error, receiver dropped before sender"))
	}
}

struct RunRx(Receiver<RunEvent>);

impl RunRx {
	/// Receives an event asynchronously.
	pub async fn recv(&self) -> Result<RunEvent> {
		self.0
			.recv_async()
			.await
			.map_err(|_| crate::Error::custom("RunQueue receive async error, sender dropped before receiver"))
	}
}

// endregion: --- RunTx/Rx
