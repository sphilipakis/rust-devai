use super::AppTx;
use super::event::AppEvent;
use crate::Result;
use crate::event::{Rx, Tx, new_channel};
use derive_more::{Deref, From};
use std::pin::Pin;
use tokio::task::JoinHandle;
use tokio::time::{Duration, Sleep};

#[derive(Clone, From, Deref)]
pub struct PingTimerTx(Tx<i64>);

// region:    --- API

pub fn start_ping_timer(app_tx: AppTx) -> Result<PingTimerTx> {
	let (tx, rx) = new_channel::<i64>("ping_timer");

	let _handle = run_ping_timer(rx, app_tx);

	Ok(PingTimerTx::from(tx))
}

// endregion: --- API

// region:    --- Worker

fn run_ping_timer(rx: Rx<i64>, app_tx: AppTx) -> JoinHandle<()> {
	tokio::spawn(async move {
		let mut pending_ts: Option<i64> = None;
		let mut sleep_fut: Option<Pin<Box<Sleep>>> = None;

		loop {
			// If we have a timer scheduled, wait on both the timer and the next ping.
			if let Some(sleep) = sleep_fut.as_mut() {
				tokio::select! {
					_ = sleep.as_mut() => {
						// Timer fired - if we have a pending ts, send one Tick and clear the pending.
						if let Some(ts) = pending_ts.take() {
							let _ = app_tx.send(AppEvent::Tick(ts)).await;
						}
						// Clear the timer; we will only schedule a new one when a new ping arrives.
						sleep_fut = None;
					}
					msg = rx.recv() => {
						match msg {
							Ok(ts) => {
								// Always keep only the most recent timestamp.
								pending_ts = Some(ts);
								// Keep the current timer (debounce window).
							}
							Err(_) => {
								// Channel closed, terminate the worker.
								break;
							}
						}
					}
				}
			} else {
				// No timer scheduled - block on the next ping.
				match rx.recv().await {
					Ok(ts) => {
						pending_ts = Some(ts);
						// Schedule the debounce timer only when we have something to send.
						sleep_fut = Some(Box::pin(tokio::time::sleep(Duration::from_millis(100))));
					}
					Err(_) => {
						// Channel closed, terminate the worker.
						break;
					}
				}
			}
		}
	})
}

// endregion: --- Worker
