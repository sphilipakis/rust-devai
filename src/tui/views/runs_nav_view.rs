use crate::support::text::format_time_local;
use crate::tui::AppState;
use crate::tui::styles::{CLR_BKG_GRAY_DARKER, CLR_BKG_SEL, CLR_TXT, CLR_TXT_GREEN, CLR_TXT_SEL, STL_TXT};
use crate::tui::support::RectExt;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget as _};

pub struct RunsNavView {}

impl StatefulWidget for RunsNavView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Render background
		Block::new().render(area, buf);

		// -- Enter Items
		let runs = state.runs();
		let items: Vec<ListItem> = runs
			.iter()
			.enumerate()
			.map(|(i, run)| {
				let (mark_txt, mark_color) = if run.is_done() {
					("✔", CLR_TXT_GREEN)
				} else {
					("▶", CLR_TXT)
				};

				let label = if let Some(start) = run.start
					&& let Ok(start_fmt) = format_time_local(start.into())
				{
					start_fmt
				} else {
					format!("Run {i}")
				};

				// TODO: need to try to avoid clone
				let label = run.label.clone().unwrap_or(label);
				let line = Line::from(vec![
					Span::styled(mark_txt, Style::default().fg(mark_color)),
					Span::styled(" ", Style::default()),
					Span::styled(label, STL_TXT),
				]);
				ListItem::new(line)
			})
			.collect();
		let list_w = List::new(items)
			.highlight_style(Style::default().bg(CLR_BKG_SEL).fg(CLR_TXT_SEL))
			.highlight_spacing(HighlightSpacing::WhenSelected);
		let mut list_s = ListState::default();
		list_s.select(state.run_idx());

		StatefulWidget::render(list_w, area.x_margin(1), buf, &mut list_s);
	}
}
