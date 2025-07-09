use crate::store::rt_model::TaskState;
use crate::tui::styles;
use crate::tui::support::{RectExt, num_pad_for_len};
use crate::tui::{AppState, TaskView};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget as _};

/// Renders the *Tasks* tab (tasks list and content).
pub struct RunDetailsView {}

impl StatefulWidget for RunDetailsView {
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
	Block::new().bg(styles::CLR_BKG_GRAY_DARKER).render(area, buf);

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

			let (mark_txt, mark_style) = match task.state() {
				TaskState::Waiting => ("⏸", styles::CLR_TXT_WAITING),
				TaskState::Running => ("▶", styles::CLR_TXT_RUNNING),
				TaskState::Done => ("✔", styles::CLR_TXT_DONE),
			};

			let line = Line::from(vec![
				Span::styled(mark_txt, mark_style),
				Span::styled(" ", Style::default()),
				Span::styled(label, styles::STL_TXT),
			]);
			ListItem::new(line)
		})
		.collect();

	// -- Create List Widget & State
	let list_w = List::new(items)
		.highlight_style(Style::default().bg(styles::CLR_BKG_SEL).fg(styles::CLR_TXT_SEL))
		.highlight_spacing(HighlightSpacing::Always);

	let mut list_s = ListState::default();
	list_s.select(state.task_idx());

	StatefulWidget::render(list_w, area.x_margin(1), buf, &mut list_s);
}
