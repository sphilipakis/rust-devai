use crate::tui::styles;
use crate::tui::support::RectExt;
use crate::tui::views::support;
use crate::tui::{AppState, TaskView};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget as _};

/// Renders the *Tasks* tab (tasks list and content).
pub struct RuntTasksView;

impl StatefulWidget for RuntTasksView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- If not tasks, render no tasks ui
		if state.tasks().is_empty() {
			render_no_tasks(area, buf, state);
			return;
		}

		// -- Render Task(s)
		let show_tasks_nav = state.tasks().len() > 1;

		let tasks_nav_width = if show_tasks_nav { 20 } else { 0 };
		let [nav_a, content_a] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Max(tasks_nav_width), Constraint::Min(0)])
			.spacing(1)
			.areas(area);

		// -- Render tasks nav
		if show_tasks_nav {
			render_tasks_nav(nav_a, buf, state);
		}

		// -- Render task content
		TaskView.render(content_a, buf, state);
	}
}

fn render_no_tasks(area: Rect, buf: &mut Buffer, state: &AppState) {
	let area = area.x_h_margin(1);
	// For now, if no Run, do not render anything
	let Some(err_id) = state.current_run().and_then(|r| r.end_err_id) else {
		Paragraph::new("No err_id for this run.").render(area, buf);
		return;
	};

	let lines = support::ui_for_err(state.mm(), err_id, area.width.min(120));

	Paragraph::new(lines).render(area, buf);
}

fn render_tasks_nav(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Render background
	Block::new().bg(styles::CLR_BKG_GRAY_DARKER).render(area, buf);

	// -- Layout before_all | Logs
	let [tasks_label_a, tasks_a] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Length(1), // tasks label
			Constraint::Fill(1),   // tasks list
		])
		.areas(area);

	// region:    --- Render Tasks Label
	let before_line = Line::default().spans(vec![
		// ..
		Span::raw(" Tasks:"),
	]);
	before_line.style(styles::STL_FIELD_LBL).render(tasks_label_a, buf);
	// endregion: --- Render Tasks Label

	// region:    --- Render Tasks

	let tasks = state.tasks();
	let tasks_len = tasks.len();
	let items: Vec<ListItem> = tasks
		.iter()
		.map(|task| {
			let line = Line::from(task.ui_label(tasks_len));
			ListItem::new(line)
		})
		.collect();

	// -- Create List Widget & State
	let list_w = List::new(items)
		.highlight_style(styles::STL_NAV_ITEM_HIGHLIGHT)
		.highlight_spacing(HighlightSpacing::Always);

	let mut list_s = ListState::default();
	list_s.select(state.task_idx());

	StatefulWidget::render(list_w, tasks_a, buf, &mut list_s);

	// endregion: --- Render Tasks
}
