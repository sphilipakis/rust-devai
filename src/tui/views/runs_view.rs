use crate::store::rt_model::LogBmc;
use crate::support::text::format_time_local;
use crate::tui::AppState;
use crate::tui::styles::{CLR_BKG_GRAY_DARKER, CLR_BKG_SEL, CLR_TXT, CLR_TXT_GREEN, CLR_TXT_SEL, STL_TXT};
use crate::tui::support::RectExt;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget};

pub struct RunsView {}

impl StatefulWidget for RunsView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- Layout Nav | Content
		// Empty line on top
		let [_, area] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![Constraint::Length(1), Constraint::Fill(1)])
			.areas(area);

		let [nav_a, content_a] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![Constraint::Max(20), Constraint::Fill(1)])
			.spacing(1)
			.areas(area);

		// -- Render nav
		self.render_nav(nav_a, buf, state);

		// -- Display the Content block
		self.render_content(content_a, buf, state);
	}
}

impl RunsView {
	fn render_nav(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
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

	fn render_content(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
		Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Draw content
		let logs = if let Some(current_run) = state.current_run() {
			LogBmc::list_for_display(state.mm(), current_run.id)
		} else {
			Ok(Vec::new())
		};

		let mut items: Vec<ListItem> = vec![];
		match logs {
			Ok(logs) => {
				for log in logs {
					if let Some(msg) = log.message {
						// let item = Paragraph::new(message).wrap(Wrap { trim: true });
						items.push(ListItem::new(format!("{} - {} - {msg}", log.run_id, log.id)))
					} else {
						let msg = log
							.step
							.map(|s| s.to_string())
							.unwrap_or_else(|| format!("No msg or step for log id {}", log.id));
						// let item = Paragraph::new(message).wrap(Wrap { trim: true });
						items.push(ListItem::new(format!("{} - {} - {msg}", log.run_id, log.id)))
					}
				}
			}
			Err(err) => items.push(ListItem::new(format!("LogBmc::list error. {err}"))),
		}
		// let p = Paragraph::new(content).wrap(Wrap { trim: true });

		let list_w = List::new(items);

		let mut list_s = ListState::default();
		StatefulWidget::render(list_w, area.x_margin(1), buf, &mut list_s);
	}
}
