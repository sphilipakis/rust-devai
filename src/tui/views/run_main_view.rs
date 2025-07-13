use crate::tui::support::{RectExt, clamp_idx_in_len};
use crate::tui::views::{RunAfterAllView, RunBeforeAllView, RuntTasksView};
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Tabs, Widget as _};

pub struct RunMainView;

pub enum RunTab {
	BeforeAll,
	Tasks,
	AfterAll,
}

impl StatefulWidget for RunMainView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		Block::new().bg(styles::CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Layout Header | Tabs | Tab Content
		let [header_a, _space_1, tabs_a, tabs_line, tab_content_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(2), // header
				Constraint::Max(1),    // space_1
				Constraint::Length(1), // tabs
				Constraint::Max(1),    // tab_line
				Constraint::Fill(1),   // tab_content
			])
			.areas(area);

		// -- render header
		render_header(header_a, buf, state);

		// -- Render tabs with line
		let selected_tab = render_tabs(tabs_a, tabs_line, buf, state);

		// -- Render the selected tab
		match selected_tab {
			RunTab::BeforeAll => {
				RunBeforeAllView.render(tab_content_a, buf, state);
			}
			RunTab::Tasks => {
				RuntTasksView.render(tab_content_a, buf, state);
			}
			RunTab::AfterAll => {
				RunAfterAllView.render(tab_content_a, buf, state);
			}
		}
	}
}

fn render_header(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Prepare Data
	let agent_name = state.current_run_agent_name();
	let model_name = state.tasks_cummulative_models();
	let cost_txt = state.current_run_cost_fmt();
	let concurrency_txt = state.current_run_concurrency_txt();

	// Tasks progress and optional cumulative duration.
	let total_tasks = state.tasks().len();
	let done_tasks = state.tasks().iter().filter(|t| t.is_done()).count();
	let tasks_txt = format!("{done_tasks}/{total_tasks}");

	let mut duration_txt = state.current_run_duration_txt();
	if let Some(cumul_txt) = state.tasks_cummulative_duration() {
		duration_txt = format!("{duration_txt} ({cumul_txt})");
	}

	// -- Layout Helpers
	let [lbl_1, val_1, lbl_2, val_2, lbl_3, val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(11), // Agent/Model
			Constraint::Length(26), //
			Constraint::Length(8),  // Tasks/Cost
			Constraint::Length(9),  //
			Constraint::Length(13), // Concurrency/Duration
			Constraint::Fill(1),    //
		])
		.spacing(1)
		.areas(area);

	// -- Render Row 1
	// Agent label with marker
	let mut line_1 = Line::default();
	if state.current_run().map(|v| v.is_done()).unwrap_or_default() {
		line_1.push_span(Span::styled("✔", styles::CLR_TXT_DONE));
	} else {
		line_1.push_span(Span::styled("▶", styles::CLR_TXT_RUNNING));
	};
	line_1.push_span(" Agent:");
	Paragraph::new(line_1)
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_1.x_row(1), buf);
	// Agent value
	Paragraph::new(agent_name)
		.style(styles::STL_FIELD_VAL)
		.render(val_1.x_row(1), buf);

	Paragraph::new("Tasks:")
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_2.x_row(1), buf);
	Paragraph::new(tasks_txt)
		.style(styles::STL_FIELD_VAL)
		.render(val_2.x_row(1), buf);

	Paragraph::new("Concurrency:")
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_3.x_row(1), buf);
	Paragraph::new(concurrency_txt)
		.style(styles::STL_FIELD_VAL)
		.render(val_3.x_row(1), buf);

	// -- Render Row 2
	Paragraph::new("Model:")
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_1.x_row(2), buf);
	Paragraph::new(model_name)
		.style(styles::STL_FIELD_VAL)
		.render(val_1.x_row(2), buf);

	Paragraph::new("Cost:")
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_2.x_row(2), buf);
	Paragraph::new(cost_txt)
		.style(styles::STL_FIELD_VAL)
		.render(val_2.x_row(2), buf);

	Paragraph::new("Duration:")
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_3.x_row(2), buf);
	Paragraph::new(duration_txt)
		.style(styles::STL_FIELD_VAL)
		.render(val_3.x_row(2), buf);
}

fn render_tabs(tabs_a: Rect, tabs_line_a: Rect, buf: &mut Buffer, state: &mut AppState) -> RunTab {
	let style = styles::stl_tab_dft();
	let highlight_style = styles::stl_tab_act();

	// -- Render tabs
	let tasks_label = if state.tasks().len() > 1 { " Tasks " } else { " Task " };

	let titles = vec![
		//
		Line::styled(" Before All ", style),
		Line::styled(tasks_label, style),
		Line::styled(" After All ", style),
	];

	// Clamp the index
	state.set_run_tab_idx(clamp_idx_in_len(state.run_tab_idx(), titles.len()));

	Tabs::new(titles)
		.highlight_style(highlight_style)
		.select(state.run_tab_idx() as usize)
		.padding(" ", "")
		.divider("")
		.render(tabs_a, buf);

	// -- Render Line
	// Trick to have a single line of tab active bkg color
	let repeated = "▔".repeat(tabs_line_a.width as usize);
	let line = Line::default().spans(vec![Span::raw(repeated)]).fg(styles::CLR_BKG_TAB_ACT);
	line.render(tabs_line_a, buf);

	// - Return tab selected
	match state.run_tab_idx() {
		0 => RunTab::BeforeAll,
		1 => RunTab::Tasks,
		2 => RunTab::AfterAll, // Fallback
		_ => RunTab::Tasks,
	}
}
