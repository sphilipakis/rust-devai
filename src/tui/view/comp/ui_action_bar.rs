use crate::tui::AppState;
use crate::tui::core::UiAction;
use crate::tui::style;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Paragraph, Widget as _};

// region:    --- Types

pub struct ActionBarBtn {
	pub label: String,
	pub shortcut: KeyCode,
	pub action: UiAction,
	pub extra_keys: Vec<KeyCode>,
}

// endregion: --- Types

// region:    --- UI Builders

/// Renders a standard two-button action bar (A on left, B on right).
/// Usually A is the "negative/cancel" action and B is the "positive/confirm" action.
pub fn ui_action_bar_ab(
	area: Rect,
	buf: &mut Buffer,
	state: &mut AppState,
	btn_a: ActionBarBtn,
	btn_b: ActionBarBtn,
) {
	let [area_a, _gap, area_b] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![Constraint::Fill(1), Constraint::Length(2), Constraint::Fill(1)])
		.areas(area);

	let label_a = format!(" [{}] {} ", key_code_to_str(&btn_a.shortcut), btn_a.label);
	let label_b = format!(" [{}] {} ", key_code_to_str(&btn_b.shortcut), btn_b.label);

	// -- Process mouse events
	if let Some(mouse_evt) = state.mouse_evt()
		&& mouse_evt.is_up()
	{
		if mouse_evt.is_over(area_a) {
			state.set_action(btn_a.action.clone());
		} else if mouse_evt.is_over(area_b) {
			state.set_action(btn_b.action.clone());
		}
	}

	// -- Process keyboard shortcuts
	if let Some(key_code) = state.last_app_event().as_key_code() {
		let is_a = *key_code == btn_a.shortcut || btn_a.extra_keys.contains(key_code);
		let is_b = *key_code == btn_b.shortcut || btn_b.extra_keys.contains(key_code);

		if is_a {
			state.set_action(btn_a.action.clone());
		} else if is_b {
			state.set_action(btn_b.action.clone());
		}
	}

	// -- Render buttons
	let stl_btn = style::STL_TAB_DEFAULT;
	let stl_btn_hover = style::STL_TAB_DEFAULT_HOVER;

	let style_a = if state.is_last_mouse_over(area_a) {
		stl_btn_hover
	} else {
		stl_btn
	};
	let style_b = if state.is_last_mouse_over(area_b) {
		stl_btn_hover
	} else {
		stl_btn
	};

	Paragraph::new(label_a)
		.alignment(Alignment::Center)
		.style(style_a)
		.render(area_a, buf);
	Paragraph::new(label_b)
		.alignment(Alignment::Center)
		.style(style_b)
		.render(area_b, buf);
}

fn key_code_to_str(key: &KeyCode) -> String {
	match key {
		KeyCode::Char(c) => c.to_string().to_uppercase(),
		KeyCode::Esc => "Esc".to_string(),
		KeyCode::Enter => "Enter".to_string(),
		_ => format!("{key:?}"),
	}
}

// endregion: --- UI Builders
