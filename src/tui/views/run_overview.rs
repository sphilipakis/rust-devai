use crate::tui::AppState;
use crate::tui::support::RectExt;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget as _};

/// Placeholder view for *Before All* tab.
pub struct RunOverviewView;

impl StatefulWidget for RunOverviewView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let area = area.x_h_margin(1);

		// -- Render Body
		render_body(area, buf, state);
	}
}

fn render_body(area: Rect, buf: &mut Buffer, state: &AppState) {
	let mut all_lines: Vec<Line> = Vec::new();

	// -- Add before all
	all_lines.extend(ui_for_before_all(state));

	// -- Render all
	Paragraph::new(all_lines).render(area, buf);
}

// region:    --- UI Builders
#[allow(clippy::vec_init_then_push)]
fn ui_for_before_all(state: &AppState) -> Vec<Line<'static>> {
	let Some(run) = state.current_run() else {
		return Default::default();
	};

	let mut all_lines: Vec<Line> = Vec::new();

	all_lines.push("--- Before All ".into());

	all_lines
}

// endregion: --- UI Builders
