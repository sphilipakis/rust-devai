//!
//! Cancellation helpers built on top of a lightweight broadcast primitive.
//! Provides lightweight wrappers so the rest of the codebase can work with a
//! cohesive `CancelTx` / `CancelRx` API similar to the other channel helpers.
//!
//! ## Design Points
//!
//! - Reuses a shared `Notify` plus a generation counter so every cancel signal advances the generation.
//! - Each `CancelRx` tracks its own `last_seen` generation, yielding only on fresh cancellations even when handles are cloned.
//! - The async wait path (`cancelled`) touches the `Notify` mutex only while registering waiters, whereas the hot check path only loads atomics.
//!
//! ## Design Considerations
//!
//! - Tokio `CancellationToken` keeps the cancelled state forever, so reusing the same token across runs would surface stale cancellations.
//! - Tokio broadcast channels add per-event allocations and lag handling and still require manual reset logic to avoid inherited cancels.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Notify;

/// Create a new cancellation token pair.
/// - `name` - Static identifier used for diagnostics and tracing.
pub fn new_cancel_trx(name: &'static str) -> CancelTrx {
	let inner = Arc::new(CancelInner::new(name));

	CancelTrx(CancelTx::new(inner.clone()), CancelRx::new(inner))
}

#[derive(Clone)]
pub struct CancelTrx(CancelTx, CancelRx);

impl CancelTrx {
	pub fn tx(&self) -> &CancelTx {
		&self.0
	}
	pub fn rx(&self) -> &CancelRx {
		&self.1
	}
}

impl fmt::Debug for CancelTrx {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("CancelTrx").field("name", &self.0.name()).finish()
	}
}

// region:    --- CancelTx

#[derive(Clone)]
pub struct CancelTx(Arc<CancelInner>);

impl CancelTx {
	fn new(inner: Arc<CancelInner>) -> Self {
		Self(inner)
	}

	/// Signal cancellation. This is idempotent.
	pub fn cancel(&self) {
		self.0.next_generation();
	}

	/// Returns `true` if cancellation has already been signalled at least once.
	pub fn is_cancelled(&self) -> bool {
		self.0.generation() > 0
	}

	/// Create a new receiver bound to the same underlying broadcast.
	pub fn subscribe(&self) -> CancelRx {
		CancelRx::new(self.0.clone())
	}

	pub fn name(&self) -> &'static str {
		self.0.name
	}
}

impl fmt::Debug for CancelTx {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("CancelTx")
			.field("name", &self.name())
			.field("generation", &self.0.generation())
			.finish()
	}
}

// endregion: --- CancelTx

// region:    --- CancelRx

#[allow(unused)]
pub struct CancelRx {
	inner: Arc<CancelInner>,
	last_seen: AtomicU64,
}

impl CancelRx {
	fn new(inner: Arc<CancelInner>) -> Self {
		let generation = inner.generation();
		Self {
			inner,
			last_seen: AtomicU64::new(generation),
		}
	}

	/// Future that resolves once cancellation is requested.
	pub async fn cancelled(&self) {
		loop {
			let current = self.inner.generation();
			let last_seen = self.last_seen.load(Ordering::SeqCst);

			if current > last_seen {
				self.last_seen.store(current, Ordering::SeqCst);
				return;
			}

			self.inner.notified().await;
		}
	}

	/// Returns `true` if cancellation has already been signalled since the last await.
	pub fn is_cancelled(&self) -> bool {
		self.inner.generation() > self.last_seen.load(Ordering::SeqCst)
	}

	pub fn name(&self) -> &'static str {
		self.inner.name
	}
}

impl Clone for CancelRx {
	fn clone(&self) -> Self {
		Self::new(self.inner.clone())
	}
}

impl fmt::Debug for CancelRx {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("CancelRx")
			.field("name", &self.name())
			.field("last_seen", &self.last_seen.load(Ordering::SeqCst))
			.finish()
	}
}

// endregion: --- CancelRx

// region:    --- CancelInner

struct CancelInner {
	name: &'static str,
	notify: Notify,
	generation: AtomicU64,
}

impl CancelInner {
	fn new(name: &'static str) -> Self {
		Self {
			name,
			notify: Notify::new(),
			generation: AtomicU64::new(0),
		}
	}

	fn generation(&self) -> u64 {
		self.generation.load(Ordering::SeqCst)
	}

	fn next_generation(&self) {
		self.generation.fetch_add(1, Ordering::SeqCst);
		self.notify.notify_waiters();
	}

	async fn notified(&self) {
		self.notify.notified().await;
	}
}

// endregion: --- CancelInner
