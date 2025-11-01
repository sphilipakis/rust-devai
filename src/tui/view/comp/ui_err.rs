use crate::model::ErrBmc;
use crate::model::{Id, ModelManager};
use crate::tui::core::{Action, LinkZones};
use crate::tui::style;
use crate::tui::view::comp;
use ratatui::text::{Line, Span};

#[allow(unused)]
pub fn ui_for_err(mm: &ModelManager, err_id: Id, max_width: u16) -> Vec<Line<'static>> {
	let marker_txt = "Error:";
	let marker_style = style::STL_SECTION_MARKER_ERR;
	let spans_prefix = vec![Span::styled("┃ ", style::CLR_TXT_RED)];
	match ErrBmc::get(mm, err_id) {
		Ok(err_rec) => {
			let content = err_rec.content.unwrap_or_default();
			let content = if let Some(stage) = err_rec.stage {
				format!("Error at stage {stage}:\n{content}")
			} else {
				content
			};
			comp::ui_for_marker_section_str(&content, (marker_txt, marker_style), max_width, Some(&spans_prefix))
		}
		Err(err) => comp::ui_for_marker_section_str(
			&format!("Error getting error. {err}"),
			(marker_txt, marker_style),
			max_width,
			None,
		),
	}
}

pub fn ui_for_err_with_hover(
	mm: &ModelManager,
	err_id: Id,
	max_width: u16,
	link_zones: &mut LinkZones,
) -> Vec<Line<'static>> {
	let marker_txt = "Error:";
	let marker_style = style::STL_SECTION_MARKER_ERR;
	let spans_prefix = vec![Span::styled("┃ ", style::CLR_TXT_RED)];

	match ErrBmc::get(mm, err_id) {
		Ok(err_rec) => {
			let content = err_rec.content.unwrap_or_default();
			let content = if let Some(stage) = err_rec.stage {
				format!("Error at stage {stage}:\n{content}")
			} else {
				content
			};

			let lines =
				comp::ui_for_marker_section_str(&content, (marker_txt, marker_style), max_width, Some(&spans_prefix));

			let mut out: Vec<Line<'static>> = Vec::new();

			// Grouped zones across the whole Error section.
			let gid = link_zones.start_group();
			for (i, line) in lines.into_iter().enumerate() {
				let span_len = line.spans.len();
				if span_len > 0 {
					let span_start = span_len - 1;
					link_zones.push_group_zone(i, span_start, 1, gid, Action::ToClipboardCopy(content.clone()));
				}
				out.push(line);
			}

			// Separator line (no zones)
			out.push(Line::default());

			out
		}
		Err(err) => {
			let content = format!("Error getting error. {err}");
			let lines = comp::ui_for_marker_section_str(&content, (marker_txt, marker_style), max_width, None);

			let mut out: Vec<Line<'static>> = Vec::new();
			let gid = link_zones.start_group();
			for (i, line) in lines.into_iter().enumerate() {
				let span_len = line.spans.len();
				if span_len > 0 {
					let span_start = span_len - 1;
					link_zones.push_group_zone(i, span_start, 1, gid, Action::ToClipboardCopy(content.clone()));
				}
				out.push(line);
			}
			out.push(Line::default());

			out
		}
	}
}
