//! HTML Utilities
use crate::{Error, Result};

/// Strips non-content elements from the provided HTML content and returns the cleaned HTML as a string.
///
/// This function removes:
/// - Non-visible tags such as `<script>`, `<link>`, `<style>`, and `<svg>`.
/// - HTML comments.
/// - Empty lines.
/// - Attributes except for `class`, `aria-label`, and `href`.
///
/// # Arguments
///
/// * `html_content` - A `String` containing the HTML content to be processed.
///
/// # Returns
///
/// A `Result<String>` which is:
/// - `Ok(String)` containing the cleaned HTML content.
/// - `Err` if any parsing or serialization errors occur.
///
pub fn slim(html_content: String) -> Result<String> {
	let res = html_helpers::slim(&html_content).map_err(|err| Error::cc("Cannot slim HTML", err))?;
	Ok(res)
}

pub fn decode_html_entities(content: &str) -> String {
	html_escape::decode_html_entities(content).to_string()
}
