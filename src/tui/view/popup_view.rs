use crate::tui::AppState;
use crate::tui::style;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::text::Text;
use ratatui::widgets::{Block, Clear, Padding, Paragraph, StatefulWidget, Widget as _};
use std::time::Duration;

// region:    --- Types

#[derive(Debug, Clone)]
pub enum PopupMode {
	/// Disappears automatically after the given duration.
	Timed(Duration),

	/// Stays on screen until dismissed by the user (Esc or click 'x').
	#[allow(unused)]
	User,
}

#[derive(Debug, Clone)]
pub struct PopupView {
	pub content: String,
	pub mode: PopupMode,
}

// endregion: --- Types

// region:    --- Overlay Widget

/// Renders the current popup (if any) centered over the UI.
pub struct PopupOverlay;

impl StatefulWidget for PopupOverlay {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let Some(popup) = state.popup().cloned() else {
			return;
		};

		// Compute a centered rect based on content.
		let lines: Vec<&str> = popup.content.lines().collect();
		let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

		// Some sane min/max for popup sizing.
		let inner_width = longest.clamp(20, area.width.saturating_sub(6)).saturating_add(2);
		let inner_height = (lines.len() as u16).clamp(1, area.height.saturating_sub(6)).saturating_add(2);

		// Build a centered area with Layout
		let [_, mid_v, _] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Fill(1),
				Constraint::Length(inner_height.saturating_add(2)), // +2 for borders
				Constraint::Fill(1),
			])
			.areas(area);

		let [_, popup_a, _] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![
				Constraint::Fill(1),
				Constraint::Length(inner_width.saturating_add(3)), // +2 for borders
				Constraint::Fill(1),
			])
			.areas(mid_v);

		// Clear only the popup area so its interior is solid and does not bleed underlying content.
		Clear.render(popup_a, buf);

		// Text style
		// TODO: need to make it property (for now, hardcode to clip)
		let txt_style = style::CLR_TXT_HOVER_TO_CLIP;

		// Render the popup content with a bordered block and black background inside the popup only.
		let para = Paragraph::new(Text::from(popup.content.clone()))
			.style(txt_style)
			.block(
				Block::bordered()
					.border_style(style::CLR_TXT_WHITE)
					.padding(Padding::new(2, 2, 1, 1))
					.bg(style::CLR_BKG_BLACK),
			)
			.centered();
		para.render(popup_a, buf);

		// If user-mode, draw an 'x' at top-right and allow click to close.
		if matches!(popup.mode, PopupMode::User) {
			// Position the 'x' one cell left from the top-right corner to keep it inside the border.
			let x_area = Rect {
				x: popup_a.x.saturating_add(popup_a.width.saturating_sub(2)),
				y: popup_a.y,
				width: 1,
				height: 1,
			};

			Paragraph::new("x").render(x_area, buf);

			// Process click on 'x'
			if let Some(mouse_evt) = state.mouse_evt()
				&& mouse_evt.is_up()
				&& mouse_evt.is_over(x_area)
			{
				state.clear_popup();
				state.trigger_redraw();
				state.clear_mouse_evts();
			}
		}
	}
}

// endregion: --- Overlay Widget
