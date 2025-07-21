//! UI For Runtime Model
//!

use crate::store::rt_model::{ErrBmc, Log, LogKind};
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

pub fn ui_for_log(log: &Log, max_width: u16) -> Vec<Line<'static>> {
	let Some(kind) = log.kind else {
		return vec![Line::raw(format!("Log [{}] has no kind", log.id))];
	};
	let content = match (log.message.as_ref(), log.kind.as_ref()) {
		(_, Some(LogKind::RunStep)) => log.step_as_str(),
		(Some(msg), _) => msg,
		(_, _) => "No Step not MSG for log",
	};

	let marker_txt_style = match kind {
		LogKind::RunStep => ("Sys Step", style::STL_SECTION_MARKER),
		LogKind::SysInfo => ("Sys Info", style::STL_SECTION_MARKER),
		LogKind::SysWarn => ("Sys Warn", style::STL_SECTION_MARKER),
		LogKind::SysError => ("Sys Error", style::STL_SECTION_MARKER),
		LogKind::SysDebug => ("Sys Debug", style::STL_SECTION_MARKER),
		LogKind::AgentPrint => ("Print:", style::STL_SECTION_MARKER),
		LogKind::AgentSkip => ("■ Skip:", style::STL_SECTION_MARKER_SKIP),
	};

	super::ui_for_marker_section_str(content, marker_txt_style, max_width, None)
}
