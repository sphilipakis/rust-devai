use crate::model::ErrBmc;
use crate::model::{Id, ModelManager};
use crate::tui::core::{UiAction, LinkZones};
use crate::tui::style;
use crate::tui::view::comp;
use ratatui::style::Color;
use ratatui::text::{Line, Span};

#[allow(unused)]
pub fn ui_for_err(mm: &ModelManager, err_id: Id, max_width: u16, path_color: Option<Color>) -> Vec<Line<'static>> {
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
			comp::ui_for_marker_section_str(
				&content,
				(marker_txt, marker_style),
				max_width,
				Some(&spans_prefix),
				None,
				None,
				path_color,
			)
		}
		Err(err) => comp::ui_for_marker_section_str(
			&format!("Error getting error. {err}"),
			(marker_txt, marker_style),
			max_width,
			None,
			None,
			None,
			path_color,
		),
	}
}

pub fn ui_for_err_with_hover(
	mm: &ModelManager,
	err_id: Id,
	max_width: u16,
	link_zones: &mut LinkZones,
	path_color: Option<Color>,
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

			let mut out = comp::ui_for_marker_section_str(
				&content,
				(marker_txt, marker_style),
				max_width,
				Some(&spans_prefix),
				Some(link_zones),
				Some(UiAction::ToClipboardCopy(content.clone())),
				path_color,
			);

			// Separator line (no zones)
			out.push(Line::default());
			link_zones.inc_current_line_by(1);

			out
		}
		Err(err) => {
			let content = format!("Error getting error. {err}");
			let mut out = comp::ui_for_marker_section_str(
				&content,
				(marker_txt, marker_style),
				max_width,
				None,
				Some(link_zones),
				Some(UiAction::ToClipboardCopy(content.clone())),
				path_color,
			);
			out.push(Line::default());
			link_zones.inc_current_line_by(1);

			out
		}
	}
}
