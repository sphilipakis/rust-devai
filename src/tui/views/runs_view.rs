use crate::store::ModelManager;
use crate::store::rt_model::{LogBmc, RunBmc};
use crate::support::text::format_time_local;
use crate::tui::event::LastAppEvent;
use crate::tui::styles::{CLR_BKG_GRAY_DARKER, CLR_BKG_SEL, CLR_TXT, CLR_TXT_GREEN, CLR_TXT_SEL, STL_TXT};
use crate::tui::support::RectExt;
use crossterm::event::KeyCode;
use modql::filter::ListOptions;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap};

pub struct RunsView {
	mm: ModelManager,
	last_event: LastAppEvent,
}

#[derive(Default)]
pub struct RunsState {
	run_idx: Option<i32>,
}

impl RunsView {
	#[allow(clippy::new_without_default)]
	pub fn new(mm: ModelManager, last_event: LastAppEvent) -> Self {
		RunsView { mm, last_event }
	}
}

impl StatefulWidget for RunsView {
	type State = RunsState;

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
		self.render_content(content_a, buf);
	}
}

impl RunsView {
	fn render_nav(&self, area: Rect, buf: &mut Buffer, state: &mut RunsState) {
		Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

		let runs = RunBmc::list(&self.mm, Some(ListOptions::from_order_bys("!id"))).unwrap_or_default();

		// -- Process state
		let offset: i32 = match self.last_event.as_key_code() {
			Some(KeyCode::Up) => -1,
			Some(KeyCode::Down) => 1,
			_ => 0,
		};
		state.run_idx = match state.run_idx {
			None => Some(0),
			Some(n) => Some((n + offset).max(0).min(runs.len() as i32 - 1)),
		};

		// -- Render background
		Block::new().render(area, buf);

		// -- Enter Items
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
		list_s.select(state.run_idx.map(|v| v as usize));

		StatefulWidget::render(list_w, area.x_margin(1), buf, &mut list_s);
	}

	fn render_content(&self, area: Rect, buf: &mut Buffer) {
		Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Draw content
		let mut content: Vec<String> = vec![];
		let list_options = ListOptions::from_order_bys("!id");
		match LogBmc::list(&self.mm, Some(list_options)) {
			Ok(logs) => {
				for message in logs.into_iter().filter_map(|v| v.message) {
					content.push(message)
				}
			}
			Err(err) => content.push(format!("LogBmc::list error. {err}")),
		}
		let content = content.join("\n\n");

		let p = Paragraph::new(content).wrap(Wrap { trim: true });

		p.render(area, buf);
	}
}
