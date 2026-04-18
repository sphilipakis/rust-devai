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
			Err(err) => tracing::warn!("AIPACK INTERNAL WARNING - failed to send event to hub - {err}"),
		}
	}

	pub fn publish_sync(&self, event: impl Into<HubEvent>) {
		match self.tx.send_sync(event) {
			Ok(_) => (),
			Err(err) => tracing::warn!("AIPACK INTERNAL WARNING - failed to send event to hub - {err}"),
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

	pub fn publish_rt_model_change_sync(&self) {
		self.publish_sync(HubEvent::RtModelChange);
	}
}

static HUB: LazyLock<Hub> = LazyLock::new(Hub::new);

pub fn get_hub() -> &'static Hub {
	&HUB
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::model::{EntityAction, EntityType, ModelEvent, RelIds};

	#[tokio::test]
	async fn test_hub_publish_model_event_simple() -> Result<()> {
		// -- Setup & Fixtures
		let hub = Hub::new();
		let rx = hub.take_rx()?;
		let model_event = ModelEvent {
			entity: EntityType::Run,
			action: EntityAction::Created,
			id: Some(42.into()),
			rel_ids: RelIds::default(),
		};

		// -- Exec
		hub.publish(model_event).await;
		let event = rx.recv().await?;

		// -- Check
		match event {
			HubEvent::Data(evt) => {
				assert_eq!(evt.entity, EntityType::Run);
				assert_eq!(evt.action, EntityAction::Created);
				assert_eq!(evt.id, Some(42.into()));
				assert_eq!(evt.rel_ids, RelIds::default());
			}
			_ => return Err("Should receive HubEvent::Data".into()),
		}

		Ok(())
	}
}

// endregion: --- Tests
