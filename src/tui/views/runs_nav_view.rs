use crate::support::text::format_time_local;
use crate::tui::support::RectExt;
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget as _};

pub struct RunsNavView;

impl StatefulWidget for RunsNavView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		Block::new().bg(styles::CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Render background
		Block::new().render(area, buf);

		// -- Render the panel label
		let [label_a, list_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Length(1), Constraint::Fill(1)])
			.spacing(1)
			.areas(area);

		Paragraph::new(" Runs: ")
			.style(styles::STL_TXT_LABEL)
			.left_aligned()
			.render(label_a.x_width(7), buf);

		// -- Get the List Items
		let runs = state.runs();
		let items: Vec<ListItem> = runs
			.iter()
			.enumerate()
			.map(|(i, run)| {
				let (mark_txt, mark_color) = if run.is_done() {
					("✔", styles::CLR_TXT_GREEN)
				} else {
					("▶", styles::CLR_TXT)
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
					Span::raw(" "),
					Span::styled(mark_txt, Style::default().fg(mark_color)),
					Span::raw(" "),
					Span::styled(label, styles::STL_TXT),
				]);
				ListItem::new(line)
			})
			.collect();

		// -- Create List Widget & State
		let list_w = List::new(items)
			.highlight_style(Style::default().bg(styles::CLR_BKG_SEL).fg(styles::CLR_TXT_SEL))
			.highlight_spacing(HighlightSpacing::WhenSelected);

		let mut list_s = ListState::default();
		list_s.select(state.run_idx());

		StatefulWidget::render(list_w, list_a, buf, &mut list_s);
	}
}
