use crate::tui::support::clamp_idx_in_len;
use crate::tui::views::{RunDetailsView, RunOverviewView};
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Tabs, Widget as _};

pub struct RunMainView;

pub enum RunTab {
	Overview,
	Details,
}

impl StatefulWidget for RunMainView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		Block::new().bg(styles::CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Layout Header | Logs
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

		// -- render top
		render_top(header_a, buf, state);

		// --
		let selected_tab = render_tabs(tabs_a, tabs_line, buf, state);

		match selected_tab {
			RunTab::Overview => {
				RunOverviewView.render(tab_content_a, buf, state);
			}
			RunTab::Details => {
				RunDetailsView.render(tab_content_a, buf, state);
			}
		}
	}
}

// region:    --- Render Helpers

fn render_top(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Prepare Data
	let agent_name = state.current_run_agent_name();
	let model_name = state.tasks_cummulative_models();
	let cost_txt = state.current_run_cost_txt();
	let concurrency_txt = state.current_run_concurrency_txt();
	let duration_txt = state.current_run_duration_txt();

	// Tasks progress and optional cumulative duration.
	let total_tasks = state.tasks().len();
	let done_tasks = state.tasks().iter().filter(|t| t.is_done()).count();
	let mut tasks_txt = format!("{done_tasks}/{total_tasks}");
	if let Some(cumul_txt) = state.tasks_cummulative_duration() {
		tasks_txt = format!("{tasks_txt} ({cumul_txt})");
	}

	// -- Layout Helpers
	// 6 columns: label / value repeated 3 times.
	let cols = vec![
		Constraint::Length(10), // "Agent:" / "Model:" labels
		Constraint::Length(20), // Values for Agent / Model
		Constraint::Length(10), // "Duration:" / "Cost:"
		Constraint::Length(10), // Values for Duration / Cost
		Constraint::Length(13), // "Tasks:" label, Concurrency: label
		Constraint::Length(20), // Tasks value or concurrency
	];

	let [line_1_a, line_2_a] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![Constraint::Length(1), Constraint::Length(1)])
		.areas(area);

	// -- Render Line 1
	let [l1_label_1, l1_val_1, l1_label_2, l1_val_2, l1_label_3, l1_val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(cols.clone())
		.spacing(1)
		.areas(line_1_a);

	let mut line_1 = Line::default();
	if state.current_run().map(|v| v.is_done()).unwrap_or_default() {
		line_1.push_span(Span::styled("✔", styles::CLR_TXT_DONE));
	} else {
		line_1.push_span(Span::styled("▶", styles::CLR_TXT_RUNNING));
	};
	line_1.push_span(" Agent:");
	Paragraph::new(line_1)
		.style(styles::STL_TXT_LABEL)
		.right_aligned()
		.render(l1_label_1, buf);
	Paragraph::new(agent_name).style(styles::STL_TXT_VALUE).render(l1_val_1, buf);

	Paragraph::new("Duration:")
		.style(styles::STL_TXT_LABEL)
		.right_aligned()
		.render(l1_label_2, buf);
	Paragraph::new(duration_txt).style(styles::STL_TXT_VALUE).render(l1_val_2, buf);

	Paragraph::new("Tasks:")
		.style(styles::STL_TXT_LABEL)
		.right_aligned()
		.render(l1_label_3, buf);
	Paragraph::new(tasks_txt).style(styles::STL_TXT_VALUE).render(l1_val_3, buf);

	// -- Render Line 2
	let [l2_label_1, l2_val_1, l2_label_2, l2_val_2, l2_label_3, l2_val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(cols)
		.spacing(1)
		.areas(line_2_a);

	Paragraph::new("Model:")
		.style(styles::STL_TXT_LABEL)
		.right_aligned()
		.render(l2_label_1, buf);
	Paragraph::new(model_name).style(styles::STL_TXT_VALUE).render(l2_val_1, buf);

	Paragraph::new("Cost:")
		.style(styles::STL_TXT_LABEL)
		.right_aligned()
		.render(l2_label_2, buf);
	Paragraph::new(cost_txt).style(styles::STL_TXT_VALUE).render(l2_val_2, buf);

	Paragraph::new("Concurrency:")
		.style(styles::STL_TXT_LABEL)
		.right_aligned()
		.render(l2_label_3, buf);
	Paragraph::new(concurrency_txt)
		.style(styles::STL_TXT_VALUE)
		.render(l2_val_3, buf);
}

fn render_tabs(tabs_a: Rect, tabs_line_a: Rect, buf: &mut Buffer, state: &mut AppState) -> RunTab {
	let style = styles::stl_tab_dft();
	let highlight_style = styles::stl_tab_act();

	// -- Render tabs
	let titles = vec![
		//
		Line::styled(" Overview ", style),
		Line::styled(" Details ", style),
	];

	// Clamp the index
	state.run_tab_idx = clamp_idx_in_len(state.run_tab_idx, titles.len());

	Tabs::new(titles)
		.highlight_style(highlight_style)
		.select(state.run_tab_idx as usize)
		.padding(" ", "")
		.divider("")
		.render(tabs_a, buf);

	// -- Render Line
	// Trick to have a single line of tab active bkg color
	let repeated = "▔".repeat(tabs_line_a.width as usize);
	let line = Line::default().spans(vec![Span::raw(repeated)]).fg(styles::CLR_BKG_TAB_ACT);
	line.render(tabs_line_a, buf);

	// - Return tab selected
	match state.run_tab_idx {
		0 => RunTab::Overview,
		1 => RunTab::Details,
		_ => RunTab::Details, // Fallback
	}
}

// endregion: --- Render Helpers
