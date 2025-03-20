//! Wrapper on top of flume bounded event fot create a OneShot logick
//!
//! Note: The benefit over tokio one shot s that this support the send/recv sync/async

use crate::{Error, Result};
use flume::{Receiver, bounded};

/// Creates a true oneshot channel based on Flume
pub fn oneshot<T>() -> (OneShotTx<T>, OneShotRx<T>) {
	let (tx, rx) = bounded(1);
	(OneShotTx { sender: Some(tx) }, OneShotRx { receiver: rx })
}

/// Wrapper for a one-shot sender that enforces single-use behavior
#[derive(Debug)]
pub struct OneShotTx<T> {
	sender: Option<flume::Sender<T>>,
}

impl<T> OneShotTx<T> {
	/// Sends a value synchronously and consumes the sender
	pub fn send(mut self, value: T) -> Result<()> {
		if let Some(sender) = self.sender.take() {
			sender
				.send(value)
				.map_err(|err| Error::custom(format!("Fail to OneShot send message. Cause: {err}")))?;
		}
		Ok(())
	}

	/// Sends a value asynchronously and consumes the sender
	pub async fn send_async(mut self, value: T) -> Result<()>
	where
		T: Send + 'static,
	{
		if let Some(sender) = self.sender.take() {
			sender
				.send_async(value)
				.await
				.map_err(|err| Error::custom(format!("Fail to OneShot send message. Cause: {err}")))?;
		}
		Ok(())
	}
}

/// Wrapper for a one-shot receiver
pub struct OneShotRx<T> {
	receiver: Receiver<T>,
}

impl<T> OneShotRx<T> {
	/// Receives the value synchronously
	pub fn recv(self) -> Result<T> {
		let v = self.receiver.recv()?;
		Ok(v)
	}

	/// Receives the value asynchronously
	#[allow(unused)]
	pub async fn recv_async(self) -> Result<T> {
		let v = self.receiver.recv_async().await?;
		Ok(v)
	}
}
