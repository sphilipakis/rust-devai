//! UI For Runtime Model
//!

use crate::store::rt_model::{Log, LogKind};
use crate::tui::styles;
use crate::tui::views::support;
use ratatui::text::Line;

pub fn ui_for_log(log: Log, max_width: u16) -> Vec<Line<'static>> {
	let Some(kind) = log.kind else {
		return vec![Line::raw(format!("Log [{}] has no kind", log.id))];
	};
	let content = match (log.message.as_ref(), log.kind.as_ref()) {
		(_, Some(LogKind::RunStep)) => log.step_as_str(),
		(Some(msg), _) => msg,
		(_, _) => "No Step not MSG for log",
	};

	let marker_txt_style = match kind {
		LogKind::RunStep => ("Sys Step", styles::STL_SECTION_MARKER),
		LogKind::SysInfo => ("Sys Info", styles::STL_SECTION_MARKER),
		LogKind::SysWarn => ("Sys Warn", styles::STL_SECTION_MARKER),
		LogKind::SysError => ("Sys Error", styles::STL_SECTION_MARKER),
		LogKind::SysDebug => ("Sys Debug", styles::STL_SECTION_MARKER),
		LogKind::AgentPrint => ("Print:", styles::STL_SECTION_MARKER),
	};

	support::ui_for_marker_section(content, marker_txt_style, max_width, None)
}
