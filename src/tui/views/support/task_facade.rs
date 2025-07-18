use crate::store::rt_model::Task;
use crate::tui::styles;
use crate::tui::support::num_pad_for_len;
use crate::tui::views::support::{self, el_running_ico};
use ratatui::style::Color;
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

	pub fn ui_stage_statuses_spans(&self) -> Vec<Span<'static>> {
		let mut spans: Vec<Span<'static>> = Vec::new();

		// -- data
		let data_running_state = self.data_running_state();
		let ico = el_running_ico(data_running_state);
		let block_spans = style_bg_spans(vec![Span::raw(" "), ico, Span::raw(" Data ")], styles::CLR_BKG_400);
		spans.extend(block_spans);

		spans.push(Span::raw("  "));

		// -- AI
		let ai_running_state = self.ai_running_state();
		let ico = el_running_ico(ai_running_state);
		let block_spans = vec![Span::raw(" "), ico, Span::raw(" AI ")];
		let block_spans = style_bg_spans(block_spans, styles::CLR_BKG_400);
		spans.extend(block_spans);

		spans
	}
}

// region:    --- Support

fn style_bg_spans(mut spans: Vec<Span<'static>>, bg: Color) -> Vec<Span<'static>> {
	for span in spans.iter_mut() {
		span.style.bg = bg.into();
	}
	spans
}

// endregion: --- Support
