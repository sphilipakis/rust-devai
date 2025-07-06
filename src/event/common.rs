//! Application channel normalization
//! - Use flume for async/sync convenient
//! - Implement send(impl Into)
//! - Default to async and _sync suffix for sync

use crate::{Error, Result};

/// Create a new unbounded channel
/// - `name` - The static name of the channel (for better error reporting)
pub fn new_channel<T>(name: &'static str) -> (Tx<T>, Rx<T>) {
	let (tx, rx) = flume::unbounded();

	(Tx(tx, name), Rx(rx, name))
}

// region:    --- Tx

// Tx with the channel name
pub struct Tx<T>(flume::Sender<T>, &'static str);

impl<T> Tx<T> {
	/// Asynchronous send (preferred).
	pub async fn send(&self, value: impl Into<T>) -> Result<()> {
		self.0.send_async(value.into()).await.map_err(|err| Error::ChannelTx {
			name: self.1,
			cause: err.to_string(),
		})
	}

	/// Synchronous send (`_sync` suffix for clarity).
	pub fn send_sync(&self, value: impl Into<T>) -> Result<()> {
		self.0.send(value.into()).map_err(|err| Error::ChannelTx {
			name: self.1,
			cause: err.to_string(),
		})
	}
}

impl<T> Clone for Tx<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), self.1)
	}
}

// endregion: --- Tx

// region:    --- Rx

// Rx with the channel name
pub struct Rx<T>(flume::Receiver<T>, &'static str);

impl<T> Rx<T> {
	/// Asynchronous receive (preferred).
	pub async fn recv(&self) -> Result<T> {
		self.0.recv_async().await.map_err(|err| Error::ChannelRx {
			name: self.1,
			cause: err.to_string(),
		})
	}

	/// Synchronous receive (`_sync` suffix for clarity).
	#[allow(unused)]
	pub fn recv_sync(&self) -> Result<T> {
		self.0.recv().map_err(|err| Error::ChannelRx {
			name: self.1,
			cause: err.to_string(),
		})
	}

	/// Non-blocking receive.
	/// Returns `Ok(None)` if channel is empty.
	#[allow(unused)]
	pub fn try_recv(&self) -> Result<Option<T>> {
		match self.0.try_recv() {
			Ok(v) => Ok(Some(v)),
			Err(flume::TryRecvError::Empty) => Ok(None),
			Err(err @ flume::TryRecvError::Disconnected) => Err(Error::ChannelRx {
				name: self.1,
				cause: err.to_string(),
			}),
		}
	}
}

impl<T> Clone for Rx<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), self.1)
	}
}

// endregion: --- Rx
