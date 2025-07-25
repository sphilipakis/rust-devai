// file: src/event/one_shot.rs
//! One-shot channel implementation using flume::bounded(1).
//!
//! Follows the same pattern as `unbound.rs` for consistency.
//! Senders and receivers are consumed on use, as is typical for one-shot channels.
//!
//! Example wraping the Tex
//! ```rust
//! #[derive(Clone, From, Deref)]
//! pub struct RunDoneTx(OneShotTx<RunResponse>);
//! ```

use crate::{Error, Result};

/// Create a new one-shot channel.
/// - `name` - The static name of the channel (for better error reporting).
#[allow(unused)]
pub fn new_one_shot_channel<T>(name: &'static str) -> (OneShotTx<T>, OneShotRx<T>) {
	let (tx, rx) = flume::bounded(1);
	(OneShotTx(tx, name), OneShotRx(rx, name))
}

// region:    --- OneShotTx

/// OneShot Sender with channel name.
/// Consumed on send.
#[derive(Debug, Clone)]
pub struct OneShotTx<T>(flume::Sender<T>, &'static str);

impl<T> OneShotTx<T> {
	/// Asynchronous send (preferred). Consumes the sender.
	pub async fn send(self, value: impl Into<T>) -> Result<()> {
		self.0.send_async(value.into()).await.map_err(|err| Error::ChannelTx {
			name: self.1,
			cause: err.to_string(),
		})
	}

	/// Synchronous send (`_sync` suffix for clarity). Consumes the sender.
	#[allow(unused)]
	pub fn send_sync(self, value: impl Into<T>) -> Result<()> {
		self.0.send(value.into()).map_err(|err| Error::ChannelTx {
			name: self.1,
			cause: err.to_string(),
		})
	}
}

// endregion: --- OneShotTx

// region:    --- OneShotRx

/// OneShot Receiver with channel name.
/// Consumed on receive.
pub struct OneShotRx<T>(flume::Receiver<T>, &'static str);

#[allow(unused)]
impl<T> OneShotRx<T> {
	/// Asynchronous receive (preferred). Consumes the receiver.
	pub async fn recv(self) -> Result<T> {
		self.0.recv_async().await.map_err(|err| Error::ChannelRx {
			name: self.1,
			cause: err.to_string(),
		})
	}

	/// Synchronous receive (`_sync` suffix for clarity). Consumes the receiver.
	pub fn recv_sync(self) -> Result<T> {
		self.0.recv().map_err(|err| Error::ChannelRx {
			name: self.1,
			cause: err.to_string(),
		})
	}
}

// endregion: --- OneShotRx
