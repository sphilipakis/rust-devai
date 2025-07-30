use crate::store::rt_model::Pin;
use crate::tui::style;
use crate::tui::view::comp;
use crate::types::uc;
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
	let marker_style = style::STL_PIN_MARKER;

	let (label_txt, content) = if let Some(content) = pin.content.as_ref() {
		match serde_json::from_str::<uc::Marker>(content) {
			Ok(uc_marker) => (uc_marker.label, uc_marker.content),
			Err(err) => ("Pin Err:".to_string(), format!("{err}")),
		}
	} else {
		("Pin:".to_string(), "No content".to_string())
	};

	super::ui_for_marker_section_str(&content, (&label_txt, marker_style), max_width, None)
}

// region:    --- Render UI Pins

// endregion: --- Render UI Pins
