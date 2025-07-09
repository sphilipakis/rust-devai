use crate::tui::styles::{CLR_BKG_GRAY_DARKER, CLR_BKG_SEL, STL_TXT_LABEL, STL_TXT_VALUE};
use crate::tui::support::clamp_idx_in_len;
use crate::tui::views::{RunDetailsView, RunOverviewView};
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Stylize as _};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Tabs, Widget as _};

pub struct RunMainView;

pub enum RunTab {
	Overview,
	Details,
}

impl StatefulWidget for RunMainView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Layout Header | Logs
		let [header_a, _space_1, tabs_a, tab_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(2),
				Constraint::Length(1),
				Constraint::Length(1),
				Constraint::Fill(1),
			])
			.areas(area);

		render_top(header_a, buf, state);

		let selected_tab = render_tabs(tabs_a, buf, state);

		match selected_tab {
			RunTab::Overview => {
				RunOverviewView.render(tab_a, buf, state);
			}
			RunTab::Details => {
				RunDetailsView.render(tab_a, buf, state);
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

	Paragraph::new("Agent:")
		.style(STL_TXT_LABEL)
		.right_aligned()
		.render(l1_label_1, buf);
	Paragraph::new(agent_name).style(STL_TXT_VALUE).render(l1_val_1, buf);

	Paragraph::new("Duration:")
		.style(STL_TXT_LABEL)
		.right_aligned()
		.render(l1_label_2, buf);
	Paragraph::new(duration_txt).style(STL_TXT_VALUE).render(l1_val_2, buf);

	Paragraph::new("Tasks:")
		.style(STL_TXT_LABEL)
		.right_aligned()
		.render(l1_label_3, buf);
	Paragraph::new(tasks_txt).style(STL_TXT_VALUE).render(l1_val_3, buf);

	// -- Render Line 2
	let [l2_label_1, l2_val_1, l2_label_2, l2_val_2, l2_label_3, l2_val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(cols)
		.spacing(1)
		.areas(line_2_a);

	Paragraph::new("Model:")
		.style(STL_TXT_LABEL)
		.right_aligned()
		.render(l2_label_1, buf);
	Paragraph::new(model_name).style(STL_TXT_VALUE).render(l2_val_1, buf);

	Paragraph::new("Cost:")
		.style(STL_TXT_LABEL)
		.right_aligned()
		.render(l2_label_2, buf);
	Paragraph::new(cost_txt).style(STL_TXT_VALUE).render(l2_val_2, buf);

	Paragraph::new("Concurrency:")
		.style(STL_TXT_LABEL)
		.right_aligned()
		.render(l2_label_3, buf);
	Paragraph::new(concurrency_txt).style(STL_TXT_VALUE).render(l2_val_3, buf);
}

fn render_tabs(area: Rect, buf: &mut Buffer, state: &mut AppState) -> RunTab {
	let style = (Color::default(), styles::CLR_BKG_GRAY_DARK);
	let titles = vec![
		//
		Line::styled(" Overview ", style),
		Line::styled(" Details ", style),
	];

	// Clamp the index
	state.run_tab_idx = clamp_idx_in_len(state.run_tab_idx, titles.len());

	let highlight_style = (Color::default(), CLR_BKG_SEL);

	Tabs::new(titles)
		.highlight_style(highlight_style)
		.select(state.run_tab_idx as usize)
		.padding(" ", "")
		.divider("")
		.render(area, buf);

	match state.run_tab_idx {
		0 => RunTab::Overview,
		1 => RunTab::Details,
		_ => RunTab::Details, // Fallback
	}
}

// endregion: --- Render Helpers
