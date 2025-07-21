use crate::tui::core::ScrollIden;
use crate::tui::styles;
use crate::tui::support::{RectExt, clamp_idx_in_len};
use crate::tui::views::support;
use crate::tui::{AppState, TaskView};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget as _};

/// Renders the *Tasks* tab (tasks list and content).
pub struct RunTasksView;

/// Component scroll identifiers
impl RunTasksView {
	const TASKS_NAV_SCROLL_IDEN: ScrollIden = ScrollIden::TasksNav;

	const SCROLL_IDENS: &[&ScrollIden] = &[&Self::TASKS_NAV_SCROLL_IDEN];

	pub fn clear_scroll_idens(state: &mut AppState) {
		state.clear_scroll_zone_areas(Self::SCROLL_IDENS);
		TaskView::clear_scroll_idens(state);
	}
}

impl StatefulWidget for RunTasksView {
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
		// IMPORTANT: Need to display nav first,
		//            because it will process the mouse event for task selection.
		if show_tasks_nav {
			render_tasks_nav(nav_a, buf, state);
		} else {
			state.clear_scroll_zone_area(&RunTasksView::TASKS_NAV_SCROLL_IDEN);
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
	const SCROLL_IDEN: ScrollIden = RunTasksView::TASKS_NAV_SCROLL_IDEN;

	// -- Render background
	Block::new().bg(styles::CLR_BKG_GRAY_DARKER).render(area, buf);

	// -- Layout before_all | Logs
	let [tasks_label_a, tasks_list_a] = Layout::default()
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

	// -- Get data

	// -- Scroll & Select logic
	state.set_scroll_area(ScrollIden::TasksNav, tasks_list_a);

	let tasks_len = state.tasks().len();
	let scroll = state.clamp_scroll(SCROLL_IDEN, tasks_len);
	// NOTE: Need to do the process mouse (task select) before the build tasks UI to get the selection
	//       To avoid a redraw. We know the number of lines.
	// NOTE: Later, we might want to trigger a redraw like runs nav to be consistent and to avoid
	//       inconsistent states (right now, tasks nav is displayed first, so, )
	process_mouse_for_task_nav(state, tasks_list_a, scroll);

	// -- Build Tasks UI
	let tasks = state.tasks();
	let task_sel_idx = state.task_idx().unwrap_or_default();
	let all_lines: Vec<ListItem> = tasks
		.iter()
		.map(|task| {
			let mut line = Line::from(task.ui_label(area.width, tasks_len));
			if let Some(idx) = task.idx
				&& task_sel_idx == idx as usize
			{
				line = line.style(styles::STL_NAV_ITEM_HIGHLIGHT);
			};
			ListItem::new(line)
		})
		.collect();

	// -- Render with widget
	let list_w = List::new(all_lines)
		// .highlight_style(styles::STL_NAV_ITEM_HIGHLIGHT)
		.highlight_spacing(HighlightSpacing::Always);

	let mut list_s = ListState::default().with_offset(scroll as usize);
	// list_s.select(state.task_idx());

	StatefulWidget::render(list_w, tasks_list_a, buf, &mut list_s);
}

// region:    --- Mouse Processing

fn process_mouse_for_task_nav(state: &mut AppState, nav_a: Rect, offset_y: u16) {
	if let Some(mouse_evt) = state.mouse_evt()
		&& mouse_evt.is_click()
		&& mouse_evt.is_in_area(nav_a)
	{
		let new_idx = mouse_evt.y() - nav_a.y + offset_y;
		let new_idx = clamp_idx_in_len(new_idx as usize, state.tasks().len());

		state.set_task_idx(Some(new_idx));
	}
}

// endregion: --- Mouse Processing
