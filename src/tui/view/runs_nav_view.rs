use crate::support::text::{self, format_time_local};
use crate::tui::core::ScrollIden;
use crate::tui::view::comp;
use crate::tui::view::support::RectExt as _;
use crate::tui::{AppState, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget as _};

pub struct RunsNavView;

impl RunsNavView {
	const NAV_SCROLL_IDEN: ScrollIden = ScrollIden::RunsNav;

	pub fn clear_scroll_idens(state: &mut AppState) {
		state.clear_scroll_zone_area(&Self::NAV_SCROLL_IDEN);
	}
}

impl StatefulWidget for RunsNavView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		const SCROLL_IDEN: ScrollIden = RunsNavView::NAV_SCROLL_IDEN;

		Block::new().bg(style::CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Render background
		Block::new().render(area, buf);

		// -- Render the panel label
		let [label_a, list_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Length(1), Constraint::Fill(1)])
			.areas(area);

		// -- Render
		Paragraph::new(" Runs: ")
			.style(style::STL_FIELD_LBL)
			.left_aligned()
			.render(label_a, buf);

		// -- Scroll & Select logic
		state.set_scroll_area(SCROLL_IDEN, list_a);
		let runs_len = state.run_items().len();
		let scroll = state.clamp_scroll(SCROLL_IDEN, runs_len);

		// -- Process UI Event
		// NOTE: In this case (contrarery to the Tasks Nav),
		//       we will trigger a redraw, because lot of state will change.
		if process_mouse_for_run_nav(state, list_a, scroll) {
			state.trigger_redraw();
			return;
		}

		// -- Build Runs UI
		let runs = state.run_items();
		let run_sel_idx = state.run_idx().unwrap_or_default();
		let is_mouse_in_nav = state.is_last_mouse_over(list_a);
		let items: Vec<ListItem> = runs
			.iter()
			.enumerate()
			.map(|(idx, run_item)| {
				let run = run_item.run();
				let run_ico = comp::el_running_ico(run);

				let label = if let Some(_parent_id) = run_item.parent_id() {
					run.agent_name.as_deref().unwrap_or("no agent name").to_string()
				} else if let Some(start) = run.start
					&& let Ok(start_fmt) = format_time_local(start.into())
				{
					start_fmt
				} else {
					format!("Run {idx}")
				};

				let prefix = text::spaces_up_to_10(run_item.indent() + 1);

				// TODO: need to try to avoid clone
				let label = run.label.clone().unwrap_or(label);
				let mut line = Line::from(vec![
					Span::raw(prefix),
					run_ico,
					Span::raw(" "),
					Span::styled(label, style::STL_TXT),
				]);

				if run_sel_idx == idx {
					line = line.style(style::STL_NAV_ITEM_HIGHLIGHT);
				} else if is_mouse_in_nav && state.is_last_mouse_over(list_a.x_row((idx + 1) as u16 - scroll)) {
					line = line.fg(style::CLR_TXT_HOVER);
				};

				ListItem::new(line)
			})
			.collect();
		let item_count = items.len();

		// -- Create & Render List
		let list_w = List::new(items)
			//.highlight_style(styles::STL_NAV_ITEM_HIGHLIGHT)
			.highlight_spacing(HighlightSpacing::WhenSelected);

		let mut list_s = ListState::default().with_offset(scroll as usize);
		// NOTE: We handle section by hand, otherwise, the list preven scroll when selected
		// list_s.select(state.run_idx());

		StatefulWidget::render(list_w, list_a, buf, &mut list_s);

		// -- Render scroll icons
		let item_count = item_count as u16;
		if item_count - scroll > list_a.height {
			let bottom_ico = list_a.x_bottom_right(1, 1);
			comp::ico_scroll_down().render(bottom_ico, buf);
		}
		if scroll > 0 && item_count > list_a.height - scroll {
			let top_ico = list_a.x_top_right(1, 1);
			comp::ico_scroll_up().render(top_ico, buf);
		}
	}
}

// region:    --- Process UI Event

/// Note: if run state change, then,
fn process_mouse_for_run_nav(state: &mut AppState, nav_a: Rect, scroll: u16) -> bool {
	if let Some(mouse_evt) = state.mouse_evt()
		&& mouse_evt.is_up()
		&& mouse_evt.is_over(nav_a)
	{
		// NOTE: Here not using clamp_idx_in_len to fix issue about last item selected
		//       when clicking on end of nav panel
		let current_run_idx = state.run_idx();

		let new_idx = mouse_evt.y() - nav_a.y + scroll;
		let runs_len = state.run_items().len();
		let new_idx = new_idx as usize;

		if new_idx >= runs_len {
			return false;
		}

		if Some(new_idx) != current_run_idx {
			state.set_run_idx(Some(new_idx));
			return true;
		}
	}
	false
}

// endregion: --- Process UI Event
