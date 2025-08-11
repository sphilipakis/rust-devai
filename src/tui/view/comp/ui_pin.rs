use crate::store::rt_model::Pin;
use crate::tui::core::{Action, LinkZones};
use crate::tui::style;
use crate::tui::view::comp;
use crate::types::uc;
use ratatui::text::Line;

/// NOTE: Add empty line after each log section
#[allow(unused)]
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

/// Build pins UI and attach LinkZones to create section-wide hover/click for each pin.
/// Clicking any line of a pin section copies the pin's content to the clipboard.
pub fn ui_for_pins_with_hover<'a>(
	pins: impl IntoIterator<Item = &'a Pin>,
	max_width: u16,
	link_zones: &mut LinkZones,
) -> Vec<Line<'static>> {
	let mut all_lines: Vec<Line<'static>> = Vec::new();

	for pin in pins {
		let lines = comp::ui_for_pin(pin, max_width);
		let base_idx = all_lines.len();

		// Extract the raw content to copy on click.
		// If uc::Marker parsing fails, copy the displayed error text; if no content, say "No content".
		let raw_content: String = if let Some(raw) = pin.content.as_ref() {
			match serde_json::from_str::<uc::Marker>(raw) {
				Ok(marker) => marker.content,
				Err(err) => format!("Pin Err: {err}"),
			}
		} else {
			"No content".to_string()
		};

		// Group the whole section so hover highlights all of it.
		let gid = link_zones.start_group();

		for (i, line) in lines.into_iter().enumerate() {
			let span_len = line.spans.len();
			if span_len > 0 {
				// Target the last span (content) per wrapped line.
				let span_start = span_len - 1;
				link_zones.push_group_zone(
					base_idx + i,
					span_start,
					1,
					gid,
					Action::ToClipboardCopy(raw_content.clone()),
				);
			}

			all_lines.push(line);
		}

		// Add a separator line (no zones attached).
		all_lines.push(Line::default());
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
