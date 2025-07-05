use crate::tui::event::LastAppEvent;
use crate::tui::styles::{CLR_BKG_ACT, CLR_BKG_GRAY_DARKER, STL_TXT, STL_TXT_ACT};
use crate::tui::support::RectExt;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};

pub struct SumView {}

#[allow(unused)]
#[derive(Default)]
pub struct SumState {
	last_app_event: LastAppEvent,
}

impl StatefulWidget for SumView {
	type State = SumState;

	fn render(self, area: Rect, buf: &mut Buffer, _state: &mut SumState) {
		// -- Layout
		let [current_a, total_a, config_a] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![Constraint::Max(40), Constraint::Max(33), Constraint::Length(8)])
			.spacing(1)
			.areas(area);

		render_current(current_a, buf);
		render_total(total_a, buf);
		render_config(config_a, buf);
	}
}

fn render_current(area: Rect, buf: &mut Buffer) {
	Block::new().bg(CLR_BKG_ACT).render(area, buf);

	let [status_a, metrics_a] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![Constraint::Min(28), Constraint::Fill(1)])
		.spacing(1)
		.areas(area);

	// -- Render status
	let status_a_inner = status_a.x_h_margin(1);
	let line_1 = Line::from(vec![
		//
		Span::styled("▶ Running pro@coder ...", STL_TXT_ACT),
	]);
	let line_2 = Line::from(vec![
		Span::styled("  Tasks: ", STL_TXT_ACT),
		Span::styled("3", STL_TXT_ACT.green().bold()),
		Span::styled("/", STL_TXT_ACT),
		Span::styled("12", STL_TXT_ACT.bold()),
		Span::styled(" (2 running)", STL_TXT.dark_gray()),
	]);
	let text = Text::from(vec![line_1, line_2]);
	Paragraph::new(text).render(status_a_inner, buf);

	// -- Render Time
	let metrics_a_inner = metrics_a.x_h_margin(1);
	let line_1 = Line::from(vec![
		//
		Span::styled("1m 12s", STL_TXT),
	]);
	let line_2 = Line::from(vec![
		//
		Span::styled("$0.012", STL_TXT),
	]);

	let text = Text::from(vec![line_1, line_2]);
	Paragraph::new(text).right_aligned().render(metrics_a_inner, buf);
}

fn render_total(area: Rect, buf: &mut Buffer) {
	Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

	let content_a = area.x_h_margin(1);

	// -- Render status
	let line_1 = Line::from(vec![
		//
		Span::styled("Total Runs", STL_TXT),
	]);
	let line_2 = Line::from(vec![
		Span::styled("Runs: ", STL_TXT),
		Span::styled("3", STL_TXT.bold()),
		Span::styled(" (1)", STL_TXT.dark_gray()),
	]);
	let text = Text::from(vec![line_1, line_2]);
	Paragraph::new(text).render(content_a, buf);

	// -- Render Time
	let line_1 = Line::from(vec![
		//
		Span::styled("30m 12s", STL_TXT),
	]);
	let line_2 = Line::from(vec![
		//
		Span::styled("$3.412", STL_TXT),
	]);
	let text = Text::from(vec![line_1, line_2]);
	Paragraph::new(text).right_aligned().render(content_a, buf);
}

fn render_config(area: Rect, buf: &mut Buffer) {
	Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

	let line_1 = Line::from(vec![
		//
		Span::styled("CONFIG", STL_TXT),
	]);
	let line_2 = Line::from(vec![
		Span::styled("b", STL_TXT),
		Span::styled(" ", STL_TXT),
		Span::styled("w", STL_TXT),
		Span::styled(" ", STL_TXT),
		Span::styled("v", STL_TXT),
	]);
	let text = Text::from(vec![line_1, line_2]);
	Paragraph::new(text).centered().render(area, buf);
}
