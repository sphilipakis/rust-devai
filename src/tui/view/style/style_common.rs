use crate::tui::view::style;
use ratatui::style::{Color, Style};

/// Returns the style for a path segment, optionally hovered or with a debug color.
pub fn style_text_path(hovered: bool, debug_color: Option<Color>) -> Style {
	let mut st = if hovered {
		style::STL_TXT_PATH_HOVER
	} else {
		style::STL_TXT_PATH
	};

	if let Some(color) = debug_color {
		st = st.fg(color);
	}

	st
}
