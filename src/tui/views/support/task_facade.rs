use crate::store::rt_model::Task;
use crate::support::text;
use crate::tui::styles;
use crate::tui::support::{StylerExt as _, num_pad_for_len};
use crate::tui::views::support::{self, el_running_ico};
use ratatui::style::Stylize as _;
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

	pub fn ui_label(&self, tasks_len: usize) -> Vec<Span<'static>> {
		let label_fmt = self.fmt_label(tasks_len);
		vec![
			Span::raw(" "),
			support::el_running_ico(self),
			Span::raw(" "),
			Span::styled(label_fmt, styles::STL_TXT),
		]
	}

	pub fn ui_sum_spans(&self) -> Vec<Span<'static>> {
		let mut all_spans: Vec<Span<'static>> = Vec::new();

		// -- Input
		if let Some(input_short) = self.input_short.as_ref() {
			const MAX: usize = 12;
			let input = text::truncate_with_ellipsis(input_short, MAX, "..");
			let input = format!("{input:<width$}", width = MAX + 3);
			let spans = vec![
				//
				Span::styled(" Input: ", styles::STL_SECTION_MARKER_INPUT),
				Span::raw(input).bg(styles::CLR_BKG_400),
			];
			all_spans.extend(spans);

			all_spans.push(Span::raw("  ")); // spacing for next
		}

		// -- data
		let data_running_state = self.data_running_state();
		let ico = el_running_ico(data_running_state);
		let block_spans = vec![Span::raw(" "), ico, Span::raw(" Data ")];
		all_spans.extend(block_spans.x_bg(styles::CLR_BKG_400));

		all_spans.push(Span::raw("  "));

		// -- AI
		let ai_running_state = self.ai_running_state();
		let ico = el_running_ico(ai_running_state);
		let block_spans = vec![Span::raw(" "), ico, Span::raw(" AI ")];
		all_spans.extend(block_spans.x_bg(styles::CLR_BKG_400));

		all_spans
	}
}
