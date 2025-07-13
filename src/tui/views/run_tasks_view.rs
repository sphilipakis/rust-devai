use crate::store::rt_model::TaskState;
use crate::tui::styles;
use crate::tui::support::num_pad_for_len;
use crate::tui::views::{RunAfterAllView, RunBeforeAllView};
use crate::tui::{AppState, TaskView};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget as _};

/// Renders the *Tasks* tab (tasks list and content).
pub struct RuntTasksView;

impl StatefulWidget for RuntTasksView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let [nav_a, content_a] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Max(20), Constraint::Min(0)])
			.spacing(1)
			.areas(area);

		// -- Render tasks nav
		render_tasks_nav(nav_a, buf, state);

		// -- Render task content
		if state.before_all_show() {
			RunBeforeAllView.render(content_a, buf, state);
		} else if state.after_all_show() {
			RunAfterAllView.render(content_a, buf, state);
		} else {
			TaskView.render(content_a, buf, state);
		}
	}
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
	before_line.style(styles::STL_TXT_LBL).render(tasks_label_a, buf);
	// endregion: --- Render Tasks Label

	// region:    --- Render Tasks

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
				Span::raw(" "),
				Span::styled(mark_txt, mark_style),
				Span::raw(" "),
				Span::styled(label, styles::STL_TXT),
			]);
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
