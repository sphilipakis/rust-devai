use crate::store::rt_model::Task;
use crate::store::{EndState, RunningState};
use crate::support::text;
use crate::tui::style;
use crate::tui::support::UiExt as _;
use crate::tui::view::comp::{self, el_running_ico};
use ratatui::style::{Style, Stylize as _};
use ratatui::text::Span;

impl Task {
	pub fn fmt_label(&self, tasks_len: usize) -> String {
		let num = text::num_pad_for_len(self.idx.unwrap_or_default(), tasks_len);
		if let Some(label) = self.label.as_ref() {
			format!("{num} - {label}")
		} else {
			format!("Task-{num}")
		}
	}

	pub fn ui_label(&self, prefix: Option<&'static str>, width: u16, tasks_len: usize) -> Vec<Span<'static>> {
		// base spans with the running icon
		let mut spans: Vec<Span<'static>> = Vec::new();
		if let Some(prefix) = prefix {
			spans.push(Span::raw(prefix));
		}
		spans.push(comp::el_running_ico(self));
		spans.push(Span::raw(" "));

		// compute & add label text and width
		let label = self.fmt_label(tasks_len);
		let width = (width - spans.x_width()) as usize;
		let label = format!("{label:<width$}");
		spans.push(Span::styled(label, style::STL_TXT));

		spans
	}

	pub fn ui_input(&self, width: u16) -> Vec<Span<'static>> {
		let mut spans = vec![
			Span::styled(" Input:", style::STL_SECTION_MARKER_INPUT),
			Span::styled(" ", style::STL_SECTION_MARKER_INPUT), // gap
		];

		let (input_text, style) = match self.input_short.as_deref() {
			Some(input_short) => (input_short, Style::new().bg(style::CLR_BKG_400)),
			None => ("No input", Style::new().bg(style::CLR_BKG_400)),
		};

		let content_width = width.saturating_sub(spans.x_width()) as usize;
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
			RunningState::NotScheduled | RunningState::Waiting => style::STL_SECTION_MARKER,
			RunningState::Ended(Some(EndState::Cancel)) => style::STL_SECTION_MARKER,
			_ => style::STL_SECTION_MARKER_AI,
		};
		let spans = vec![
			//
			Span::raw(" "),
			ico.fg(style::CLR_TXT_500),
			Span::styled(" AI ", label_style),
		];

		spans.x_bg(style::CLR_BKG_400)
	}

	pub fn ui_output(&self, width: u16) -> Vec<Span<'static>> {
		let mut spans = vec![
			Span::styled("  Output:", style::STL_SECTION_MARKER_OUTPUT),
			Span::styled(" ", style::STL_SECTION_MARKER_OUTPUT), // gap
		];

		let (output_text, style) = match self.output_short.as_deref() {
			Some(output_short) => (output_short, Style::new().bg(style::CLR_BKG_400)),
			None => {
				//("No output", Style::new().bg(style::CLR_BKG_400))
				return Vec::new();
			}
		};

		let content_width = width.saturating_sub(spans.x_width()) as usize;
		let output_text = text::truncate_with_ellipsis(output_text, content_width - 2, "..");
		let output_text = output_text.replace("\n", " ");

		let output_text = format!("{output_text:<content_width$}");

		spans.push(Span::styled(output_text, style));

		spans
	}

	pub fn ui_skip(&self, width: u16) -> Vec<Span<'static>> {
		if self.has_skip() {
			let mut spans = vec![
				Span::styled(" Skipped:", style::STL_SECTION_MARKER_SKIP),
				Span::styled(" ", style::STL_SECTION_MARKER_SKIP), // gap
			];

			let (content, style) = match self.end_skip_reason.as_deref() {
				Some(reason) => (reason, Style::new().bg(style::CLR_BKG_400)),
				None => ("Task was skipped by Lua code", Style::new().bg(style::CLR_BKG_400)),
			};

			let content_width = width.saturating_sub(spans.x_width()) as usize;
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

/// For the ShortBlock facade
impl Task {
	pub fn ui_short_block(&self, max_num: usize) -> Vec<Span<'static>> {
		let num = text::num_pad_for_len(self.idx.unwrap_or_default(), max_num);
		// let mut running_ico = el_running_ico(self);
		// running_ico = running_ico.style(style::CLR_TXT_WHITE);
		let mut spans = vec![
			//
			Span::raw(" "),
			// running_ico,
			// Span::raw(" "),
			Span::raw(num),
			Span::raw(" "),
		];

		let bg = match RunningState::from(self) {
			RunningState::NotScheduled | RunningState::Unknown => style::CLR_BKG_RUNNING_WAIT,
			RunningState::Waiting => style::CLR_BKG_RUNNING_WAIT,
			RunningState::Running => {
				if self.is_ai_running() {
					style::CLR_BKG_RUNNING_AI
				} else {
					style::CLR_BKG_RUNNING_OTHER
				}
			}
			RunningState::Ended(end_state) => match end_state {
				Some(EndState::Ok) => style::CLR_BKG_RUNNING_DONE,
				Some(EndState::Err) => style::CLR_BKG_RUNNING_ERR,
				Some(EndState::Skip) => style::CLR_BKG_RUNNING_SKIP,
				Some(EndState::Cancel) => style::CLR_BKG_RUNNING_OTHER,
				None => style::CLR_BKG_RUNNING_OTHER,
			},
		};
		// spans.x_bg(bg).x_fg(style::CLR_TXT_RED)
		for span in spans.iter_mut() {
			span.style.bg = Some(bg);
			span.style.fg = Some(style::CLR_TXT_BLACK);
		}

		spans
	}
}
