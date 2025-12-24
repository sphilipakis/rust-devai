use crate::tui::core::{UiAction, AppState, LinkZones};
use crate::tui::style;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Stylize as _};
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

		// -- Build action spans and link zones
		let mut all_spans: Vec<Span> = Vec::new();
		let mut link_zones = LinkZones::default();

		// Helper to push an action item
		let push_action = |spans: &mut Vec<Span>, zones: &mut LinkZones, key: &str, label: &str, action: UiAction| {
			let span_start = spans.len();
			spans.push(Span::raw("["));
			spans.push(Span::styled(key.to_string(), style::STL_TXT_ACTION));
			spans.push(Span::raw(label.to_string()));
			let span_end = spans.len();
			zones.push_link_zone(0, span_start, span_end - span_start, action);
		};

		push_action(&mut all_spans, &mut link_zones, "r", "] Replay  ", UiAction::Redo);
		push_action(
			&mut all_spans,
			&mut link_zones,
			"x",
			"] Cancel Run  ",
			UiAction::CancelRun,
		);
		push_action(&mut all_spans, &mut link_zones, "q", "] Quit  ", UiAction::Quit);
		push_action(&mut all_spans, &mut link_zones, "n", n_label, UiAction::ToggleRunsNav);

		all_spans.push(Span::raw("  "));

		let overview_mode = state.overview_tasks_mode().to_string();
		push_action(
			&mut all_spans,
			&mut link_zones,
			"t",
			&format!("] Tasks overview: {overview_mode}  "),
			UiAction::CycleTasksOverviewMode,
		);

		let mut line = Line::from(all_spans);

		// -- Handle mouse hover and click
		let zones = link_zones.into_zones();
		let mut hovered_idx: Option<usize> = None;
		for (i, zone) in zones.iter().enumerate() {
			if zone
				.is_mouse_over(actions_a, 0, state.last_mouse_evt(), &mut line.spans)
				.is_some()
			{
				hovered_idx = Some(i);
				break;
			}
		}

		if let Some(i) = hovered_idx {
			let action = zones[i].action.clone();
			if let Some(hover_spans) = zones[i].spans_slice_mut(&mut line.spans) {
				for span in hover_spans {
					span.style = span.style.add_modifier(Modifier::BOLD).fg(style::CLR_TXT_ACTION);
				}
			}

			if state.is_mouse_up_only() && state.is_last_mouse_over(actions_a) {
				state.set_action(action);
				state.trigger_redraw();
				state.clear_mouse_evts();
			}
		}

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
