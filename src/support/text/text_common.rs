//! String utils

use crate::{Error, Result};
use aho_corasick::AhoCorasick;
use derive_more::derive::Display;
use num_format::ToFormattedString;
use std::borrow::Cow;
use std::time::Duration;
use time::{OffsetDateTime, format_description};

pub fn format_num(num: i64) -> String {
	num.to_formatted_string(&num_format::Locale::en)
}

pub fn format_duration(duration: Duration) -> String {
	let duration_ms = duration.as_millis().min(u64::MAX as u128) as u64;
	let duration = if duration_ms > 6000 {
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

// already in e
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

// region:    --- Ensure

pub struct EnsureOptions {
	pub prefix: Option<String>,
	pub suffix: Option<String>,
}

pub fn ensure(s: &str, ensure_inst: EnsureOptions) -> Cow<str> {
	let mut parts: Vec<&str> = Vec::new();

	// Add start prefix if needed
	if let Some(start) = ensure_inst.prefix.as_deref() {
		if !s.starts_with(start) {
			parts.push(start);
		}
	}

	// Always include the main string
	parts.push(s);

	// Add end suffix if needed
	if let Some(end) = ensure_inst.suffix.as_deref() {
		if !s.ends_with(end) {
			parts.push(end);
		}
	}

	// If no changes were made, return the original string as borrowed
	if parts.len() == 1 {
		Cow::Borrowed(s)
	} else {
		Cow::Owned(parts.concat()) // Join parts into a single owned string
	}
}
// endregion: --- Ensure

/// And efficient way to remove the first line.
/// Returns: (first_line, remain), and empty string none
/// Note: Good when big content, and the remain will not require new allocation
pub fn extract_first_line(mut content: String) -> (String, String) {
	if let Some(pos) = content.find('\n') {
		let remainder = content.split_off(pos + 1); // Moves remainder to a new String, avoids shifting
		(content, remainder)
	} else {
		// No newline, return whole string as first line, empty remainder
		(content, String::new())
	}
}

pub fn truncate_with_ellipsis<'a>(s: &'a str, max_len: usize, ellipsis: &str) -> Cow<'a, str> {
	if s.len() > max_len {
		let truncated = &s[..max_len];
		if ellipsis.is_empty() {
			// no allocation needed
			Cow::from(truncated)
		} else {
			Cow::from(format!("{truncated}{ellipsis}"))
		}
	} else {
		Cow::from(s)
	}
}

/// Make sure that the text end with one and only one single newline
/// Useful for code sanitization
pub fn ensure_single_ending_newline(mut text: String) -> String {
	if text.is_empty() {
		text.push('\n'); // If the string is empty, just add a newline
		return text;
	}

	// Note: Some, perhaps unnecessary, optimization to avoid traversing the whole string or doing unnecessary allocation.
	let chars = text.chars().rev(); // Create an iterator that traverses the string backwards

	// Count the number of trailing newlines
	let mut newline_count = 0;
	for ch in chars {
		if ch == '\n' {
			newline_count += 1;
		} else {
			break;
		}
	}

	match newline_count {
		0 => text.push('\n'),                                 // No trailing newlines, add one
		1 => (),                                              // Exactly one newline, do nothing
		_ => text.truncate(text.len() - (newline_count - 1)), // More than one newline, remove extra
	}

	text
}

// region:    --- Replace

/// Replaces content sections delimited by specific start and end markers.
///
/// This function iterates through the input `content` line by line. It identifies
/// lines containing the `marker_start` and `marker_end`.
///
/// The line containing the `marker_start` and all subsequent lines up to and
/// including the line containing the corresponding `marker_end` are removed.
/// The line that contained the `marker_end` is then replaced by the
/// corresponding element from the `sections` slice.
///
/// # Arguments
///
/// * `content`: The input string content to process.
/// * `sections`: A slice of string slices (`&[&str]`) where each element is
///   the content to be inserted into a marker-delimited section in the output.
///   The number of elements in `sections` must match the number of marker pairs
///   found in the `content`.
/// * `marker_pair`: A tuple containing the start marker string slice (`&str`)
///   and the end marker string slice (`&str`).
///
/// # Errors
///
/// Returns an `Err` if:
///
/// * A start marker is found when already inside a section (i.e., an unclosed
///   section).
/// * An end marker is found when not currently inside a section (i.e., no
///   matching start marker).
/// * The number of end markers found does not match the number of sections
///   provided in the `sections` slice.
///
/// # Returns
///
/// Returns `Ok(String)` containing the new content with sections replaced,
/// or an `Err` if any of the error conditions are met.
///
/// # Example
///
/// ```rust
/// # use crate::aipack::support::text::text_common::replace_markers;
/// # type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.
/// # fn main() -> Result<()> {
/// let markers = &("<<START>>", "<<END>>");
/// let content = r#"
/// Before section 1
/// // <<START>> some comment
/// Content to be replaced 1
/// // <<END>> some other comment
/// Between sections
///    <<START>>
/// Content to be replaced 2
///    <<END>>  trailing whitespace
/// After section 2
/// "#;
/// let sections = &["NEW SECTION 1", "NEW SECTION 2"];
///
/// let new_content = replace_markers(content, sections, markers)?;
///
/// assert!(!new_content.contains("Content to be replaced 1"));
/// assert!(!new_content.contains("Content to be replaced 2"));
/// assert!(new_content.contains("NEW SECTION 1"));
/// assert!(new_content.contains("NEW SECTION 2"));
/// // The lines containing markers are replaced entirely, including comments and whitespace.
/// assert!(!new_content.contains("// <<START>>"));
/// assert!(!new_content.contains("// <<END>>"));
/// assert!(!new_content.contains(" some comment"));
/// assert!(!new_content.contains(" some other comment"));
/// assert!(!new_content.contains("    <<START>>"));
/// assert!(!new_content.contains("    <<END>>"));
/// assert!(!new_content.contains(" trailing whitespace"));
/// # Ok(())
/// # }
/// ```
pub fn replace_markers(content: &str, sections: &[&str], marker_pair: &(&str, &str)) -> Result<String> {
	let lines = content.lines();
	let mut section_iter = sections.iter();
	let mut new_content: Vec<&str> = Vec::new();

	let (marker_start, marker_end) = marker_pair;

	#[derive(Display)]
	enum State {
		StartMakerLine,
		InSection,
		EndMarkerLine,
		OutSection,
	}
	let mut state = State::OutSection;

	for line in lines {
		// -- compute next state
		state = if line.contains(marker_start) {
			if matches!(state, State::StartMakerLine | State::InSection) {
				return Err(format!(
					"replace_markers - Cannot have start marker {marker_start} when previous section not closed with {marker_end}"
				)
				.into());
			}
			State::StartMakerLine
		} else if line.contains(marker_end) {
			if matches!(state, State::OutSection) {
				return Err(format!(
					"replace_markers - Cannot have close marker {marker_end} when not having open with a {marker_start}"
				)
				.into());
			}
			State::EndMarkerLine
		} else {
			// compute from prevous state
			// TODO: probably need to do some check as well
			match state {
				State::StartMakerLine => State::InSection,
				State::InSection => State::InSection,
				State::EndMarkerLine => State::OutSection,
				State::OutSection => State::OutSection,
			}
		};

		// -- add to new_content
		match state {
			State::StartMakerLine => (), // Skip the start marker line
			State::InSection => (),      // Skip lines within the section
			State::EndMarkerLine => {
				// Replace the end marker line with the section content
				let section = section_iter.next().ok_or("replace_markers - Not enough matching sections")?;
				new_content.push(section);
			}
			State::OutSection => new_content.push(line), // Keep lines outside sections
		}
	}

	// make sure to add a new empty line if original ended with one
	let original_end_with_newline = content.as_bytes().last().map(|&b| b == b'\n').unwrap_or_default();
	if original_end_with_newline {
		new_content.push(""); // to have the last empty line on join("\n")
	}

	if section_iter.next().is_some() {
		return Err("replace_markers - Not all sections have been used".to_string().into());
	}

	Ok(new_content.join("\n"))
}

#[allow(unused)]
pub fn replace_all(content: &str, patterns: &[&str], values: &[&str]) -> Result<String> {
	let ac = AhoCorasick::new(patterns).map_err(|err| Error::cc("replace_all fail because patterns", err))?;

	let res = ac.replace_all_bytes(content.as_bytes(), values);
	let new_content =
		String::from_utf8(res).map_err(|err| Error::cc("replace_all fail because result is not utf8", err))?;

	Ok(new_content)
}

// endregion: --- Replace

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{assert_contains, assert_not_contains};

	#[test]
	fn test_support_text_replace_markers_simple() -> Result<()> {
		// -- Setup & Fixtures
		let markers = &("<<START>>", "<<END>>");
		let content = r#"
	This is some content-01
	// <<START>>
	with some instruction for markers. inst-01
	// <<END>>
	and some more content-02
<<START>>
	Another set of instructions here. inst-02
<<END>>	

And more content-03
"#;
		let sections = &["SECTION-01", "// SECTION 02"];

		// -- Exec
		let new_content = replace_markers(content, sections, markers)?;

		// -- Check
		assert_contains(&new_content, "content-01");
		assert_contains(&new_content, "content-02");
		assert_contains(&new_content, "content-03\n");
		assert_contains(&new_content, "SECTION-01");
		assert_contains(&new_content, "// SECTION 02");
		assert_not_contains(&new_content, "<<START>>");
		assert_not_contains(&new_content, "<<END>>");
		assert_not_contains(&new_content, "inst-01");
		assert_not_contains(&new_content, "inst-02");

		// Check that the lines with markers were replaced entirely
		assert_not_contains(&new_content, "// <<START>>");
		assert_not_contains(&new_content, "// <<END>>");
		assert_not_contains(&new_content, "<<START>>\n"); // Ensure standalone start marker line is gone
		assert_not_contains(&new_content, "<<END>>\t\n"); // Ensure standalone end marker line is gone (including trailing whitespace)

		Ok(())
	}

	#[test]
	fn test_support_text_markers_no_closing() -> Result<()> {
		// -- Setup & Fixtures
		let markers = &("<<START>>", "<<END>>");
		let content = r#"
	This is some content-01
	// <<START>>
	with some instruction for markers. inst-01
<<START>>
	Another set of instructions here. inst-02
<<END>>	

And more content-03
"#;
		let sections = &["SECTION-01", "// SECTION 02"];

		// -- Exec
		let res = replace_markers(content, sections, markers);

		// -- Check
		if res.is_ok() {
			return Err("Should have fail replace_markers because wrong content".into());
		}

		Ok(())
	}

	#[test]
	fn test_support_text_extract_first_line() -> Result<()> {
		// -- Test case 1: String with multiple lines
		let content = "First line\nSecond line\nThird line".to_string();
		let (first, remainder) = extract_first_line(content);

		assert_eq!(first, "First line\n");
		assert_eq!(remainder, "Second line\nThird line");

		// -- Test case 2: String with only one line with newline
		let content = "Single line\n".to_string();
		let (first, remainder) = extract_first_line(content);

		assert_eq!(first, "Single line\n");
		assert_eq!(remainder, "");

		// -- Test case 3: String with only one line without newline
		let content = "No newline".to_string();
		let (first, remainder) = extract_first_line(content);

		assert_eq!(first, "No newline");
		assert_eq!(remainder, "");

		// -- Test case 4: Empty string
		let content = "".to_string();
		let (first, remainder) = extract_first_line(content);

		assert_eq!(first, "");
		assert_eq!(remainder, "");

		Ok(())
	}
}

// endregion: --- Tests
