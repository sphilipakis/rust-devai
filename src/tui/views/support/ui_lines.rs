use crate::tui::styles;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use std::borrow::Cow;

pub const MARKER_WIDTH: usize = 10;

/// This is the task view record section with the marker and content, for each log line, or for input, output, (pins in the future)
/// NOTE: Probably can make Line lifetime same as content (to avoid string duplication). But since needs to be indented, probably not a big win.
pub fn ui_for_marker_section(
	content: &str,
	(marker_txt, marker_style): (&str, Style),
	max_width: u16,
	spans_prefix: Option<&Vec<Span<'static>>>,
) -> Vec<Line<'static>> {
	let spacer = " ";
	let width_spacer = spacer.len(); // won't work if no ASCII
	let width_content = (max_width as usize) - MARKER_WIDTH - width_spacer;

	// -- Mark Span
	let mark_span = Span::styled(format!("{marker_txt:>MARKER_WIDTH$}"), marker_style);

	let spaced_idented_content: Cow<str> = if content.contains("\t") {
		// 4 spaces
		Cow::Owned(content.replace("\t", "    "))
	} else {
		Cow::Borrowed(content)
	};

	let msg_wrap = textwrap::wrap(&spaced_idented_content, width_content);

	let msg_wrap_len = msg_wrap.len();

	// -- First Content Line
	let mut msg_wrap_iter = msg_wrap.into_iter();
	let first_content = msg_wrap_iter.next().unwrap_or_default();
	let first_content_span = Span::styled(first_content.to_string(), styles::STL_SECTION_TXT);

	let mut first_line = Line::from(vec![
		//
		mark_span,
		Span::raw(spacer),
	]);
	if let Some(spans_prefix) = spans_prefix {
		let spans_prefix = spans_prefix.to_vec();
		first_line.extend(spans_prefix.to_vec());
	}
	first_line.push_span(first_content_span);

	// -- Lines
	let mut lines = vec![first_line];

	// -- Render other content line if present
	if msg_wrap_len > 1 {
		let left_spacing = " ".repeat(MARKER_WIDTH + width_spacer);
		for line_content in msg_wrap_iter {
			let mut spans: Vec<Span<'static>> = vec![Span::raw(left_spacing.to_string())];
			if let Some(spans_prefix) = spans_prefix {
				let spans_prefix = spans_prefix.to_vec();
				spans.extend(spans_prefix.to_vec());
			}
			spans.push(Span::styled(line_content.into_owned(), styles::STL_SECTION_TXT));
			lines.push(spans.into())
		}
	}

	// -- Return lines
	lines
}
