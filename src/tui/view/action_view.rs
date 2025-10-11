use crate::tui::core::AppState;
use crate::tui::style;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize as _;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};

pub struct ActionView;

impl StatefulWidget for ActionView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// Block::new().render(area, buf);

		let (sys_lbl, sys_val) = if state.show_sys_states() { (5, 10) } else { (0, 0) };

		// -- layout
		let [actions_a, dbg_clr_a, mem_lbl_a, mem_val_a, db_lbl_a, db_val_a] = ratatui::layout::Layout::default()
			.direction(ratatui::layout::Direction::Horizontal)
			.constraints(vec![
				ratatui::layout::Constraint::Fill(1),         // actions
				ratatui::layout::Constraint::Length(5),       // debug_clr
				ratatui::layout::Constraint::Length(sys_lbl), // mem_lbl
				ratatui::layout::Constraint::Length(sys_val), // mem_val
				ratatui::layout::Constraint::Length(sys_lbl), // db_lbl
				ratatui::layout::Constraint::Length(sys_val), // db_val
			])
			.spacing(1)
			.areas(area);
		// For cpu
		// cpu_lbl_a, cpu_val_a
		// ratatui::layout::Constraint::Length(5),  // cpu_lbl
		// ratatui::layout::Constraint::Length(5),  // cpu_val

		let n_label = if state.show_runs() {
			"] Hide Runs Nav"
		} else {
			"] Show Runs Nav"
		};

		// -- Render Actions Bar
		let overview_mode = state.overview_tasks_mode().to_string();
		let line = Line::from(vec![
			Span::raw("["),
			Span::styled("r", style::STL_TXT_ACTION),
			Span::raw("] Replay  "),
			Span::raw("["),
			Span::styled("x", style::STL_TXT_ACTION),
			Span::raw("] Cancel Run  "),
			Span::raw("["),
			Span::styled("q", style::STL_TXT_ACTION),
			Span::raw("] Quit  "),
			Span::raw("["),
			Span::styled("n", style::STL_TXT_ACTION),
			Span::raw(n_label),
			Span::raw("  "),
			Span::raw("["),
			Span::styled("t", style::STL_TXT_ACTION),
			Span::raw(format!("] Tasks overview: {overview_mode}  ")),
		]);

		Paragraph::new(line).render(actions_a, buf);

		// -- Render debug clr
		let dbg_clr = state.debug_clr();
		if dbg_clr != 0 {
			Paragraph::new(dbg_clr.to_string())
				.fg(ratatui::style::Color::Indexed(dbg_clr))
				.render(dbg_clr_a, buf);
		}

		if state.show_sys_states() {
			// -- Render Memory
			Paragraph::new("Mem:")
				.right_aligned()
				.style(style::STL_FIELD_LBL)
				.render(mem_lbl_a, buf);
			Paragraph::new(state.memory_fmt())
				.style(style::STL_FIELD_VAL)
				.render(mem_val_a, buf);

			// -- Render DB Memory
			Paragraph::new("DB:")
				.right_aligned()
				.style(style::STL_FIELD_LBL)
				.render(db_lbl_a, buf);
			Paragraph::new(state.db_memory_fmt())
				.style(style::STL_FIELD_VAL)
				.render(db_val_a, buf);

			// -- Render CPU
			// NOTE: Probably need to / by number of cpus
			//       And have post refresh to let it go down. (perhaps when mouse event)
			// Paragraph::new("CPU:")
			// 	.right_aligned()
			// 	.style(styles::STL_TXT_LBL)
			// 	.render(cpu_lbl_a, buf);
			// Paragraph::new(state.cpu_fmt())
			// 	.right_aligned()
			// 	.style(styles::STL_TXT_VAL)
			// 	.render(cpu_val_a, buf);
		}
	}
}
