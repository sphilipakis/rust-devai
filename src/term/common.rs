/// Checks if the user's answer is "yes".
///
/// Returns true if the input matches "Y" or "YES" (case insensitive).
///
/// NOTE: This normalizes the logic for what is considered a "yes" response.
///
pub fn is_input_yes(input: &str) -> bool {
	let input = input.trim();
	input.eq_ignore_ascii_case("y") || input.eq_ignore_ascii_case("yes")
}
