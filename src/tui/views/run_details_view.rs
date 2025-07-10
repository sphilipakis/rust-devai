use crate::store::rt_model::TaskState;
use crate::tui::styles;
use crate::tui::support::num_pad_for_len;
use crate::tui::{AppState, TaskView};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget as _};

/// Renders the *Tasks* tab (tasks list and content).
pub struct RunDetailsView;

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
		TaskView.render(task_content_a, buf, state);
	}
}

fn render_tasks_nav(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Render background
	Block::new().bg(styles::CLR_BKG_GRAY_DARKER).render(area, buf);

	// -- Layout before_all | Logs
	let [before_a, _gap_1, tasks_label_a, tasks_a, _gap_2, after_a, _margin_bottom] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Length(1), // before
			Constraint::Length(1), // _gap_1
			Constraint::Length(1), // tasks label
			Constraint::Fill(1),   // tasks list
			Constraint::Length(1), // _gap_2
			Constraint::Length(1), // after
			Constraint::Length(1), // _ (margin bottom)
		])
		.areas(area);

	// region:    --- Render Before All

	let before_line = Line::default().spans(vec![
		// ..
		Span::raw(" Before All"),
	]);
	before_line.render(before_a, buf);

	// endregion: --- Render Before All

	// region:    --- Render Tasks Label
	let before_line = Line::default().spans(vec![
		// ..
		Span::raw(" Tasks:"),
	]);
	before_line.style(styles::STL_TXT_LABEL).render(tasks_label_a, buf);
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
		.highlight_style(Style::default().bg(styles::CLR_BKG_SEL).fg(styles::CLR_TXT_SEL))
		.highlight_spacing(HighlightSpacing::Always);

	let mut list_s = ListState::default();
	list_s.select(state.task_idx());

	StatefulWidget::render(list_w, tasks_a, buf, &mut list_s);

	// endregion: --- Render Tasks

	// region:    --- Render After All

	let before_line = Line::default().spans(vec![
		// ..
		Span::raw(" After All"),
	]);
	before_line.render(after_a, buf);

	// endregion: --- Render After All
}
