use crate::support::VecExt as _;
use crate::tui::core::{UiAction, LinkZones};
use crate::tui::style;
use crate::tui::support::UiExt;
use crate::tui::view::support;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::borrow::Cow;

const MARKER_MIN_WIDTH: usize = 10;

/// This is the task view record section with the marker and content, for each log line, or for input, output, (pins in the future)
/// NOTE: Probably can make Line lifetime same as content (to avoid string duplication). But since needs to be indented, probably not a big win.
pub fn ui_for_marker_section_str(
	content: &str,
	(marker_txt, marker_style): (&str, Style),
	max_width: u16,
	content_prefix: Option<&Vec<Span<'static>>>,
	mut link_zones: Option<&mut LinkZones>,
	action: Option<UiAction>,
	path_color: Option<Color>,
) -> Vec<Line<'static>> {
	let spacer = " ";
	let width_spacer = spacer.len(); // won't work if no ASCII
	let marker_width = marker_width_for_marker_txt(marker_txt);
	let width_content = (max_width as usize) - marker_width - width_spacer;

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

	// -- Prep LinkZones group
	let group_id = if let Some(lz) = link_zones.as_mut()
		&& action.is_some()
	{
		Some(lz.start_group())
	} else {
		None
	};

	// -- Lines accumulator
	let mut lines = Vec::new();

	// Helper to push a line with segments and link zones

	let mut push_line_fn = |rel_line_idx: usize,
	                        prefix_spans: Vec<Span<'static>>,
	                        line_content: &str,
	                        mut lz_opt: Option<&mut LinkZones>,
	                        gid_opt: Option<u32>,
	                        main_action_opt: Option<&UiAction>| {
		let mut spans = prefix_spans;
		let content_span_start = spans.len();

		let segments = support::segment_line_path(line_content);

		for seg in segments {
			let style = if seg.file_path.is_some() {
				style::style_text_path(false, path_color)
			} else {
				style::STL_SECTION_TXT
			};
			let span_idx = spans.len();
			spans.push(Span::styled(seg.text, style));

			if let Some(lz) = lz_opt.as_mut() {
				if let Some(path) = seg.file_path {
					lz.push_link_zone(rel_line_idx, span_idx, 1, UiAction::OpenFile(path.to_string()));
				} else if let (Some(gid), Some(act)) = (gid_opt, main_action_opt) {
					lz.push_group_zone(rel_line_idx, span_idx, 1, gid, act.clone());
				}
			}
		}

		// Register the whole content as a group zone to ensure consistent interaction coverage for the section.
		// Precedence logic (shortest span wins) ensures path-specific zones take priority when overlapped.
		if let (Some(lz), Some(gid), Some(act)) = (lz_opt, gid_opt, main_action_opt) {
			lz.push_group_zone(
				rel_line_idx,
				content_span_start,
				spans.len() - content_span_start,
				gid,
				act.clone(),
			);
		}

		lines.push(Line::from(spans));
	};

	// -- First Content Line
	let first_content = msg_wrap_iter.next().unwrap_or_default();
	let mut first_prefix = vec![mark_span, Span::raw(spacer)];
	if let Some(spans_prefix) = content_prefix {
		first_prefix.extend(spans_prefix.to_vec());
	}

	push_line_fn(
		0,
		first_prefix,
		&first_content,
		link_zones.as_deref_mut(),
		group_id,
		action.as_ref(),
	);

	// -- Render other content line if present
	if msg_wrap_len > 1 {
		let left_spacing = " ".repeat(marker_width + width_spacer);
		for (i, line_content) in msg_wrap_iter.enumerate() {
			let mut other_prefix = vec![Span::raw(left_spacing.to_string())];
			if let Some(spans_prefix) = content_prefix {
				other_prefix.extend(spans_prefix.to_vec());
			}

			push_line_fn(
				i + 1,
				other_prefix,
				&line_content,
				link_zones.as_deref_mut(),
				group_id,
				action.as_ref(),
			);
		}
	}

	// -- Update current line if link_zones present
	if let Some(lz) = link_zones.as_mut() {
		lz.inc_current_line_by(lines.len());
	}

	// -- Return lines
	lines
}

pub fn new_marker(marker_txt: &str, marker_style: Style) -> Span<'static> {
	let marker_width = marker_width_for_marker_txt(marker_txt);
	Span::styled(format!("{marker_txt:>marker_width$}"), marker_style)
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
		let marker_width = marker_width_for_marker_spans(&marker_spans);
		let left_indent = " ".repeat(marker_width);

		// add the first line
		all_lines.push(marker_spans.extended(spacer_spans.clone()).extended(first_content_spans).into());

		for spans_line in content_spans_lines_iter {
			let spans = vec![Span::raw(left_indent.to_string())]
				.extended(spacer_spans.clone())
				.extended(spans_line);
			all_lines.push(spans.into());
		}
	}

	all_lines
}

// region:    --- Support

fn marker_width_for_marker_txt(marker_txt: &str) -> usize {
	marker_txt.chars().count().max(MARKER_MIN_WIDTH)
}

fn marker_width_for_marker_spans(marker_spans: &[Span]) -> usize {
	marker_spans.x_width().max(MARKER_MIN_WIDTH as u16) as usize
}

// endregion: --- Support
