use crate::event::{Rx, Tx, new_channel};
use crate::hub::hub_event::HubEvent;
use crate::{Error, Result};
use std::fmt::Display;
use std::sync::{Arc, LazyLock, Mutex};

/// Hub for receiving and broadcasting all OutEvent to the systems.
/// Those events are Log Message, Error, and Stage(StagEvent) to capture each progress steps
pub struct Hub {
	tx: Tx<HubEvent>,
	rx_holder: Arc<Mutex<Option<Rx<HubEvent>>>>,
}

/// Core Hub Methods
impl Hub {
	pub fn new() -> Self {
		let (tx, rx) = new_channel("main_hub");

		let rx_holder = Mutex::new(Some(rx)).into();

		Self { tx, rx_holder }
	}

	pub fn take_rx(&self) -> Result<Rx<HubEvent>> {
		let mut rx_holder = self
			.rx_holder
			.lock()
			.map_err(|err| format!("Hub::take_rx fail on mutex: {err}"))?;
		let rx = rx_holder.take().ok_or("Hub Rx already taken, cannot take twice")?;

		Ok(rx)
	}
}

/// Publish event
impl Hub {
	pub async fn publish(&self, event: impl Into<HubEvent>) {
		let event = event.into();

		match self.tx.send(event).await {
			Ok(_) => (),
			Err(err) => println!("AIPACK INTERNAL ERROR - failed to send event to hub - {err}"),
		}
	}

	pub fn publish_sync(&self, event: impl Into<HubEvent>) {
		match self.tx.send_sync(event) {
			Ok(_) => (),
			Err(err) => println!("AIPACK INTERNAL ERROR - failed to send event to hub - {err}"),
		}
	}
}

/// Convenient Methods
impl Hub {
	pub async fn publish_err(&self, msg: impl Into<String>, cause: Option<impl Display>) {
		match cause {
			Some(cause) => {
				self.publish(Error::cc(msg, cause)).await;
			}
			None => self.publish(Error::Custom(msg.into())).await,
		}
	}

	pub fn publish_err_sync(&self, msg: impl Into<String>, cause: Option<impl Display>) {
		match cause {
			Some(cause) => {
				self.publish_sync(Error::cc(msg, cause));
			}
			None => self.publish_sync(Error::Custom(msg.into())),
		}
	}
}

static HUB: LazyLock<Hub> = LazyLock::new(Hub::new);

pub fn get_hub() -> &'static Hub {
	&HUB
}

// Example usage in an async context
#[cfg(test)]
mod tests {
	use super::*;

	pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	#[tokio::test]
	async fn test_hub() -> Result<()> {
		let hub = get_hub();

		let rx = hub.take_rx()?;
		tokio::spawn(async move {
			while let Ok(event) = rx.recv().await {
				#[allow(clippy::single_match)]
				match event {
					HubEvent::Message(msg) => {
						println!("Received Message: {msg}");
					}
					_ => (),
				}
			}
		});

		// Testing async publish
		hub.publish(HubEvent::Message("Hello, world!".into())).await;

		// NOTE: Call below will fail in test because require multi-thread
		// hub.publish_sync(Event::Message("Hello from sync!".to_string()));

		Ok(())
	}
}
