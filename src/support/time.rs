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
