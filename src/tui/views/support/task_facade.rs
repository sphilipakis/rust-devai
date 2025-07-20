use crate::store::RunningState;
use crate::store::rt_model::Task;
use crate::support::text;
use crate::tui::styles;
use crate::tui::support::{UiExt as _, num_pad_for_len};
use crate::tui::views::support::{self, el_running_ico};
use ratatui::style::{Style, Stylize as _};
use ratatui::text::Span;

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
		let input_text = input_text.replace("\n", " ");

		let input_text = format!("{input_text:<content_width$}");

		spans.push(Span::styled(input_text, style));

		spans
	}
	/// NOTE: This is fixed 6 char component
	pub fn ui_ai(&self) -> Vec<Span<'static>> {
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

		spans.x_bg(styles::CLR_BKG_400)
	}

	pub fn ui_output(&self, width: u16) -> Vec<Span<'static>> {
		let mut spans = vec![
			Span::styled("  Output:", styles::STL_SECTION_MARKER_OUTPUT),
			Span::styled(" ", styles::STL_SECTION_MARKER_OUTPUT), // gap
		];

		let (output_text, style) = match self.output_short.as_deref() {
			Some(output_short) => (output_short, Style::new().bg(styles::CLR_BKG_400)),
			None => ("No output", Style::new().bg(styles::CLR_BKG_400)),
		};

		let content_width = width.saturating_sub(spans.x_total_width()) as usize;
		let output_text = text::truncate_with_ellipsis(output_text, content_width - 2, "..");
		let output_text = output_text.replace("\n", " ");

		let output_text = format!("{output_text:<content_width$}");

		spans.push(Span::styled(output_text, style));

		spans
	}

	pub fn ui_skip(&self, width: u16) -> Vec<Span<'static>> {
		if self.has_skip() {
			let mut spans = vec![
				Span::styled(" Skipped:", styles::STL_SECTION_MARKER_SKIP),
				Span::styled(" ", styles::STL_SECTION_MARKER_SKIP), // gap
			];

			let (content, style) = match self.end_skip_reason.as_deref() {
				Some(reason) => (reason, Style::new().bg(styles::CLR_BKG_400)),
				None => ("Task was skipped by Lua code", Style::new().bg(styles::CLR_BKG_400)),
			};

			let content_width = width.saturating_sub(spans.x_total_width()) as usize;
			let content = text::truncate_with_ellipsis(content, content_width - 2, "..");
			let content = content.replace("\n", " ");

			let content = format!("{content:<content_width$}");

			spans.push(Span::styled(content, style));

			spans
		} else {
			Vec::new()
		}
	}
}
