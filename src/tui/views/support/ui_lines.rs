use crate::support::VecExt as _;
use crate::tui::styles;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use std::borrow::Cow;

pub const MARKER_WIDTH: usize = 10;

/// This is the task view record section with the marker and content, for each log line, or for input, output, (pins in the future)
/// NOTE: Probably can make Line lifetime same as content (to avoid string duplication). But since needs to be indented, probably not a big win.
pub fn ui_for_marker_section_str(
	content: &str,
	(marker_txt, marker_style): (&str, Style),
	max_width: u16,
	content_prefix: Option<&Vec<Span<'static>>>,
) -> Vec<Line<'static>> {
	let spacer = " ";
	let width_spacer = spacer.len(); // won't work if no ASCII
	let width_content = (max_width as usize) - MARKER_WIDTH - width_spacer;

	// -- Mark Span
	let mark_span = new_marker(marker_txt, marker_style);

	let spaced_idented_content: Cow<str> = if content.contains("\t") {
		// 4 spaces
		Cow::Owned(content.replace("\t", "    "))
	} else {
		Cow::Borrowed(content)
	};

	let msg_wrap = textwrap::wrap(&spaced_idented_content, width_content);

	let msg_wrap_len = msg_wrap.len();

	let mut msg_wrap_iter = msg_wrap.into_iter();

	// -- First Content Line
	let first_content = msg_wrap_iter.next().unwrap_or_default();
	let first_content_span = Span::styled(first_content.to_string(), styles::STL_SECTION_TXT);

	let mut first_line = Line::from(vec![
		//
		mark_span,
		Span::raw(spacer),
	]);
	if let Some(spans_prefix) = content_prefix {
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
			if let Some(spans_prefix) = content_prefix {
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

pub fn new_marker(marker_txt: &str, marker_style: Style) -> Span<'static> {
	Span::styled(format!("{marker_txt:>MARKER_WIDTH$}"), marker_style)
}

/// Will merge the content Lines with the marker spans and spacers
/// and repeat the spacer and content lines for the number of "lines" in the content_spans
pub fn ui_for_marker_section(
	marker_spans: Vec<Span<'static>>,
	spacer_spans: Vec<Span<'static>>,
	content_spans_lines: Vec<Vec<Span<'static>>>,
) -> Vec<Line<'static>> {
	let mut all_lines: Vec<Line<'static>> = Vec::new();

	// -- No content case
	if content_spans_lines.is_empty() {
		return vec![marker_spans.extended(spacer_spans).into()];
	}

	let content_len = content_spans_lines.len();
	let mut content_spans_lines_iter = content_spans_lines.into_iter();

	let Some(first_content_spans) = content_spans_lines_iter.next() else {
		return all_lines;
	};

	if content_len == 1 {
		return vec![marker_spans.extended(spacer_spans).extended(first_content_spans).into()];
	} else {
		// add the first line
		all_lines.push(marker_spans.extended(spacer_spans.clone()).extended(first_content_spans).into());
		let left_indent = " ".repeat(MARKER_WIDTH);
		for spans_line in content_spans_lines_iter {
			let spans = vec![Span::raw(left_indent.to_string())]
				.extended(spacer_spans.clone())
				.extended(spans_line);
			all_lines.push(spans.into());
		}
	}

	all_lines
}
