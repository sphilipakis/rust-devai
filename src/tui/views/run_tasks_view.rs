use crate::tui::styles::{CLR_BKG_GRAY_DARKER, CLR_BKG_SEL, CLR_TXT_GREEN, CLR_TXT_SEL, STL_TXT};
use crate::tui::support::{RectExt, num_pad_for_len};
use crate::tui::{AppState, TaskView};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget as _};

/// Renders the *Tasks* tab (tasks list and content).
pub struct RunTasksView {}

impl StatefulWidget for RunTasksView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let [tasks_nav_a, task_content_a] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Max(20), Constraint::Min(0)])
			.spacing(1)
			.areas(area);

		// -- Render tasks nav
		render_tasks_nav(tasks_nav_a, buf, state);

		// -- Render task content
		let task_content_v = TaskView {};
		task_content_v.render(task_content_a, buf, state);
	}
}

fn render_tasks_nav(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Render background
	Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

	// -- Create Items
	let tasks = state.tasks();
	let tasks_len = tasks.len();
	let items: Vec<ListItem> = tasks
		.iter()
		.enumerate()
		.map(|(idx, task)| {
			let label = task
				.label
				.clone()
				.unwrap_or_else(|| format!("task-{} ({})", num_pad_for_len(idx, tasks_len), task.id));

			let (mark_txt, mark_style) = if task.is_done() {
				("✔", Style::default().fg(CLR_TXT_GREEN))
			} else {
				("▶", STL_TXT)
			};

			let line = Line::from(vec![
				Span::styled(mark_txt, mark_style),
				Span::styled(" ", Style::default()),
				Span::styled(label, STL_TXT),
			]);
			ListItem::new(line)
		})
		.collect();

	// -- Create List Widget & State
	let list_w = List::new(items)
		.highlight_style(Style::default().bg(CLR_BKG_SEL).fg(CLR_TXT_SEL))
		.highlight_spacing(HighlightSpacing::Always);

	let mut list_s = ListState::default();
	list_s.select(state.task_idx());

	StatefulWidget::render(list_w, area.x_margin(1), buf, &mut list_s);
}
