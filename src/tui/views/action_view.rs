use crate::tui::core::AppState;
use crate::tui::styles;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};

pub struct ActionView;

impl StatefulWidget for ActionView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// Block::new().render(area, buf);

		// -- layout
		let [actions_a, mem_lbl_a, mem_val_a, cpu_lbl_a, cpu_val_a] = ratatui::layout::Layout::default()
			.direction(ratatui::layout::Direction::Horizontal)
			.constraints(vec![
				ratatui::layout::Constraint::Fill(1),    // actions
				ratatui::layout::Constraint::Length(5),  // mem_lbl
				ratatui::layout::Constraint::Length(12), // mem_val
				ratatui::layout::Constraint::Length(5),  // cpu_lbl
				ratatui::layout::Constraint::Length(6),  // cpu_val
			])
			.spacing(1)
			.areas(area);

		let n_label = if state.show_runs() {
			"] Hide Runs Nav"
		} else {
			"] Show Runs Nav"
		};

		// -- Render Actions Bar
		let line = Line::from(vec![
			Span::raw("["),
			Span::styled("r", styles::STL_TXT_ACTION),
			Span::raw("] Replay  "),
			Span::raw("["),
			Span::styled("q", styles::STL_TXT_ACTION),
			Span::raw("] Quit  "),
			Span::raw("["),
			Span::styled("n", styles::STL_TXT_ACTION),
			Span::raw(n_label),
		]);

		Paragraph::new(line).render(actions_a, buf);

		// -- Render Memory
		Paragraph::new("Mem:")
			.right_aligned()
			.style(styles::STL_TXT_LBL)
			.render(mem_lbl_a, buf);
		Paragraph::new(state.memory_fmt())
			.style(styles::STL_TXT_VAL)
			.render(mem_val_a, buf);

		// -- Render CPU
		Paragraph::new("CPU:")
			.right_aligned()
			.style(styles::STL_TXT_LBL)
			.render(cpu_lbl_a, buf);
		Paragraph::new(state.cpu_fmt())
			.style(styles::STL_TXT_VAL)
			.render(cpu_val_a, buf);
	}
}
