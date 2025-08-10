use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Returns the Unix Time in microseconds.
///
/// Note 1: If there is any error with `duration_since UNIX_EPOCH` (which should almost never happen),
///         it returns the start of the EPOCH.
/// Note 2: The maximum UTC epoch date that can be stored in i64 with microseconds precision
///         would be approximately `292277-01-09 ... UTC`.
///         Thus, for all practical purposes, it is sufficiently distant to be of no concern.
pub fn now_micro() -> i64 {
	let now = SystemTime::now();
	let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap_or(Duration::new(0, 0));
	since_the_epoch.as_micros().min(i64::MAX as u128) as i64
}

/// Returns the tick phase (0..k-1) for a repeating cycle,
/// based on current time and the tick length in seconds.
///
/// - `tick_seconds` <= 0 → clamped to 0.1
/// - `k` <= 0 → clamped to 1
pub fn tick_phase(epoch_micros: i64, tick_seconds: f64, k: i64) -> i64 {
	let safe_tick = if tick_seconds <= 0.0 { 0.1 } else { tick_seconds };
	// Ensure at least 1 microsecond per tick even for tiny tick_seconds
	let micros_per_tick = ((safe_tick * 1_000_000.0).round() as i64).max(1);

	let cycle = if k <= 0 { 1 } else { k };
	// If you never want negative epochs to produce negative phases, clamp to 0:
	let t = epoch_micros.max(0);
	(t / micros_per_tick) % cycle
}

/// Returns true when the current tick is the first in a cycle of `k` ticks.
/// Useful for periodic triggers.
///
/// Example: `every_kth_tick(now_micro(), 0.2, 2)` is true once every 400ms.
pub fn every_kth_tick(epoch_micros: i64, tick_seconds: f64, k: i64) -> bool {
	tick_phase(epoch_micros, tick_seconds, k) == 0
}
