use crate::store::rt_model::ErrBmc;
use crate::store::{Id, ModelManager};
use crate::tui::style;
use crate::tui::view::comp;
use ratatui::text::{Line, Span};

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
