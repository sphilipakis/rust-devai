use crate::Result;
use genai::chat::Usage;
use num_format::ToFormattedString;
use std::time::Duration;
use time::{OffsetDateTime, format_description};

// region:    --- Numbers

pub fn format_num(num: i64) -> String {
	num.to_formatted_string(&num_format::Locale::en)
}

pub fn float_max_dec(val: f64, max_dec: u16) -> f64 {
	let factor = 10f64.powi(max_dec as i32);
	(val * factor).round() / factor
}

pub fn format_f64(val: f64) -> String {
	let rounded = float_max_dec(val, 4);
	format!("{:.*}", 4, rounded)
}

// endregion: --- Numbers

// region:    --- Duration

pub fn format_duration(duration: Duration) -> String {
	let duration_ms = duration.as_millis().min(u64::MAX as u128) as u64;
	let duration = if duration_ms > 10000 {
		Duration::from_secs(duration.as_secs())
	} else {
		Duration::from_millis(duration_ms)
	};
	humantime::format_duration(duration).to_string()
}

pub fn format_duration_us(duration_us: i64) -> String {
	let duration = Duration::from_micros(duration_us as u64);
	format_duration(duration)
}

// already in
pub fn format_time_local(epoch_us: i64) -> Result<String> {
	fn inner(epoch_us: i64) -> std::result::Result<String, Box<dyn std::error::Error>> {
		let secs = epoch_us / 1_000_000;
		let utc_dt = OffsetDateTime::from_unix_timestamp(secs)?;
		let local_offset = OffsetDateTime::now_local()?.offset();

		let local_dt = utc_dt.to_offset(local_offset);
		// let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")?;
		let format = format_description::parse("At [hour]:[minute]:[second]")?;
		Ok(local_dt.format(&format)?)
	}

	let res = inner(epoch_us).map_err(|err| format!("Cannot format epoch_us '{epoch_us}'. Cause: {err}"))?;

	Ok(res)
}

// endregion: --- Duration

/// Formats a byte size as a pretty, fixed-width string with unit alignment.
/// The output format is tailored to align nicely in monospaced tables.
///
/// - **B, KB, MB:** The number part is 6 characters wide.
/// - **GB, TB, PB:** The number part is 5 characters wide.
/// - **Bytes (B):** Displayed as an integer with a trailing space (e.g., `"   777 B "`). Total length is 10.
/// - **KB and above:** Displayed with 2 decimal places (e.g., `"  8.78 KB"`). Total length is 9.
///
/// ### Examples
///
/// `777`       -> `"   777 B "`
/// `8777`      -> `"  8.78 KB"`
/// `88777`     -> `" 88.78 KB"`
/// `888777`    -> `"888.78 KB"`
/// `2_345_678_900` -> `" 2.35 GB"`
///
/// NOTE: if in simple-fs, migh call it pretty_size()
pub fn format_size_xfixed(size_in_bytes: u64) -> String {
	const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
	let mut size = size_in_bytes as f64;
	let mut unit = 0;

	// Determine which unit to use
	while size >= 1000.0 && unit < UNITS.len() - 1 {
		size /= 1000.0;
		unit += 1;
	}

	let unit_str = UNITS[unit];

	// Note: The logic is derived from the test cases, which have specific formatting rules.
	if unit == 0 {
		// Bytes: integer, pad to 6, then add " B "
		let number_str = format!("{size_in_bytes:>6}");
		format!("{number_str} {unit_str} ")
	} else {
		// Units KB or above: 2 decimals, pad to width, then add " unit"
		let width = if unit <= 2 { 6 } else { 5 };
		let number_str = format!("{size:>width$.2}");
		format!("{number_str} {unit_str}")
	}
}

// region:    --- Genai

/// Format the `Prompt Tokens: 2,070 | Completion Tokens: 131`
pub fn format_usage(usage: &Usage) -> String {
	let mut buff = String::new();

	buff.push_str("Prompt Tokens: ");
	buff.push_str(&format_num(usage.prompt_tokens.unwrap_or_default() as i64));
	if let Some(prompt_tokens_details) = usage.prompt_tokens_details.as_ref() {
		buff.push_str(" (cached: ");
		let cached = prompt_tokens_details.cached_tokens.unwrap_or(0);
		buff.push_str(&format_num(cached as i64));
		if let Some(cache_creation_tokens) = prompt_tokens_details.cache_creation_tokens {
			buff.push_str(", cache_creation: ");
			buff.push_str(&format_num(cache_creation_tokens as i64));
		}
		buff.push(')');
	}

	buff.push_str(" | Completion Tokens: ");
	buff.push_str(&format_num(usage.completion_tokens.unwrap_or_default() as i64));
	if let Some(reasoning) = usage.completion_tokens_details.as_ref().and_then(|v| v.reasoning_tokens) {
		buff.push_str(" (reasoning: ");
		buff.push_str(&format_num(reasoning as i64));
		buff.push(')');
	}

	buff
}

// endregion: --- Genai

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_support_text_format_size_xfixed() -> Result<()> {
		// -- Setup & Fixtures
		let cases = [
			(777, "   777 B "),
			(8777, "  8.78 KB"),
			(88777, " 88.78 KB"),
			(888777, "888.78 KB"),
			(888700, "888.70 KB"),
			(200000, "200.00 KB"),
			(2_000_000, "  2.00 MB"),
			(2_345_678_900, " 2.35 GB"),
			(1_234_567_890_123, " 1.23 TB"),
			(2_345_678_900_123_456, " 2.35 PB"),
			(0, "     0 B "),
		];

		// -- Exec
		for &(input, expected) in &cases {
			let actual = format_size_xfixed(input);
			assert_eq!(actual, expected, "input: {input}");
		}

		// -- Check

		Ok(())
	}
}

// endregion: --- Tests
