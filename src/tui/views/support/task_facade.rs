use crate::store::rt_model::Task;
use crate::store::{EndState, RunningState};
use crate::support::text;
use crate::tui::styles;
use crate::tui::support::{UiExt as _, num_pad_for_len};
use crate::tui::views::support::{self, el_running_ico};
use ratatui::style::{Style, Stylize as _};
use ratatui::text::Span;

const MAX_INPUT_CHARS: usize = 12;
const MAX_OUTPUT_CHARS: usize = 24;

impl Task {
	pub fn fmt_label(&self, tasks_len: usize) -> String {
		let num = num_pad_for_len(self.idx.unwrap_or_default(), tasks_len);
		if let Some(label) = self.label.as_ref() {
			format!("{num} - {label}")
		} else {
			format!("Task-{num}")
		}
	}

	pub fn ui_label(&self, width: u16, tasks_len: usize) -> Vec<Span<'static>> {
		// base spans with the running icon
		let mut spans = vec![Span::raw(" "), support::el_running_ico(self), Span::raw(" ")];

		// compute & add label text and width
		let label = self.fmt_label(tasks_len);
		let width = (width - spans.x_total_width()) as usize;
		let label = format!("{label:<width$}");
		spans.push(Span::styled(label, styles::STL_TXT));

		spans
	}

	pub fn ui_input(&self, width: u16) -> Vec<Span<'static>> {
		let mut spans = vec![
			Span::styled(" Input:", styles::STL_SECTION_MARKER_INPUT),
			Span::styled(" ", styles::STL_SECTION_MARKER_INPUT), // gap
		];

		let (input_text, style) = match self.input_short.as_deref() {
			Some(input_short) => (input_short, Style::new().bg(styles::CLR_BKG_400)),
			None => ("No input", Style::new().bg(styles::CLR_BKG_400)),
		};

		let content_width = width.saturating_sub(spans.x_total_width()) as usize;
		let input_text = text::truncate_with_ellipsis(input_text, content_width - 2, "..");

		let input_text = format!("{input_text:<content_width$}");

		spans.push(Span::styled(input_text, style));

		spans
	}

	pub fn ui_output(&self, width: u16) -> Vec<Span<'static>> {
		let mut spans = vec![
			Span::styled(" Output:", styles::STL_SECTION_MARKER_INPUT),
			Span::styled(" ", styles::STL_SECTION_MARKER_INPUT), // gap
		];

		let (output_text, style) = match self.output_short.as_deref() {
			Some(output_short) => (output_short, Style::new().bg(styles::CLR_BKG_400)),
			None => ("No output", Style::new().bg(styles::CLR_BKG_400)),
		};

		let content_width = width.saturating_sub(spans.x_total_width()) as usize;
		let output_text = text::truncate_with_ellipsis(output_text, content_width - 2, "..");

		let output_text = format!("{output_text:<content_width$}");

		spans.push(Span::styled(output_text, style));

		spans
	}

	pub fn ui_sum_spans(&self) -> Vec<Span<'static>> {
		let mut all_spans: Vec<Span<'static>> = Vec::new();

		// -- Input
		if let Some(input_short) = self.input_short.as_ref() {
			let input = text::truncate_with_ellipsis(input_short, MAX_INPUT_CHARS, "..");
			let input = format!("{input:<width$}", width = MAX_INPUT_CHARS + 3);
			let spans = vec![
				//
				Span::styled(" Input: ", styles::STL_SECTION_MARKER_INPUT),
				Span::raw(input).bg(styles::CLR_BKG_400),
			];
			all_spans.extend(spans);
		}

		// -- data
		// let data_running_state = self.data_running_state();
		// let ico = el_running_ico(data_running_state);
		// let block_spans = vec![Span::raw(" "), ico, Span::raw(" Data ")];
		// all_spans.extend(block_spans.x_bg(styles::CLR_BKG_400));

		// all_spans.push(Span::raw("  "));

		// -- AI
		if !all_spans.is_empty() {
			all_spans.push(Span::raw("  ")); // spacing for next
		}
		let ai_running_state = self.ai_running_state();
		let ico = el_running_ico(ai_running_state.clone());
		let label_style = match ai_running_state {
			RunningState::NotScheduled | RunningState::Waiting => styles::STL_SECTION_MARKER,
			_ => styles::STL_SECTION_MARKER_AI,
		};
		let spans = vec![
			//
			Span::raw(" "),
			ico.fg(styles::CLR_TXT_500),
			Span::styled(" AI ", label_style),
		];
		all_spans.extend(spans.x_bg(styles::CLR_BKG_400));

		// -- output
		if let Some(output_short) = self.output_short.as_ref() {
			if !all_spans.is_empty() {
				all_spans.push(Span::raw("  ")); // spacing for next
			}

			let output = text::truncate_with_ellipsis(output_short, MAX_OUTPUT_CHARS, "..");
			let output = output.replace("\n", " ");
			let output = format!("{output:<width$}", width = MAX_OUTPUT_CHARS + 3);

			let spans = vec![
				//
				Span::styled("  Output: ", styles::STL_SECTION_MARKER_OUTPUT),
				Span::raw(output).bg(styles::CLR_BKG_400),
			];
			all_spans.extend(spans);
		}

		// -- skip
		if let Some(EndState::Skip) = self.end_state.as_ref() {
			if !all_spans.is_empty() {
				all_spans.push(Span::raw("  ")); // spacing for next
			}
			let label_style = styles::STL_SECTION_MARKER_SKIP;
			let spans = if let Some(reason) = self.end_skip_reason.as_deref() {
				let reason = text::truncate_with_ellipsis(reason, MAX_OUTPUT_CHARS, "..");
				let reason = reason.replace("\n", " ");
				let reason = format!("{reason:<width$}", width = MAX_OUTPUT_CHARS + 3);
				vec![
					//
					Span::styled(" Skipped: ", label_style),
					Span::raw(reason).bg(styles::CLR_BKG_400),
				]
			} else {
				vec![
					//
					Span::styled("  Skipped", label_style),
				]
			};
			all_spans.extend(spans);
		}

		all_spans
	}
}
