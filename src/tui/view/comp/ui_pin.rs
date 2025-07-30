use crate::store::rt_model::Pin;
use crate::tui::style;
use crate::tui::view::comp;
use ratatui::text::Line;

/// NOTE: Add empty line after each log section
pub fn ui_for_pins<'a>(pins: impl IntoIterator<Item = &'a Pin>, max_width: u16) -> Vec<Line<'static>> {
	let mut all_lines: Vec<Line> = Vec::new();
	for pin in pins {
		// Render log lines
		let pin_lines = comp::ui_for_pin(pin, max_width);
		all_lines.extend(pin_lines);
		all_lines.push(Line::default()); // empty line (for now)
	}

	all_lines
}

/// Return the lines for a single log entity
pub fn ui_for_pin(pin: &Pin, max_width: u16) -> Vec<Line<'static>> {
	let marker_txt = "Pin:";
	let marker_style = style::STL_PIN_MARKER;
	let content = pin.content.as_deref().unwrap_or("No pin Content?");

	super::ui_for_marker_section_str(content, (marker_txt, marker_style), max_width, None)
}
