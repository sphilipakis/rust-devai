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

/// Number of whole ticks since `epoch_micros`.
/// e.g., 1s with 0.2s tick → 5
pub fn tick_count(time_micro: i64, tick_seconds: f64) -> i64 {
	let safe_tick = if tick_seconds <= 0.0 { 0.1 } else { tick_seconds };
	let micros_per_tick = ((safe_tick * 1_000_000.0).round() as i64).max(1);
	(time_micro.max(0)) / micros_per_tick
}

/// Tick phase in a `k`-cycle (0..k-1).
/// e.g., 0.2s tick, k=2 → alternates 0/1 every 200ms
#[allow(unused)]
pub fn tick_phase(time_micro: i64, tick_seconds: f64, k: i64) -> i64 {
	let cycle = if k <= 0 { 1 } else { k };
	tick_count(time_micro, tick_seconds) % cycle
}

/// True if this tick is the first in a `k`-cycle.
/// e.g., 0.2s tick, k=2 → true every 400ms
#[allow(unused)]
pub fn every_kth_tick(time_micro: i64, tick_seconds: f64, k: i64) -> bool {
	tick_phase(time_micro, tick_seconds, k) == 0
}
