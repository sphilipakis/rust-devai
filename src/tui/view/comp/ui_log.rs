use crate::store::Stage;
use crate::store::rt_model::{Log, LogKind};
use crate::tui::style;
use crate::tui::view::comp;
use ratatui::text::Line;

/// NOTE: Add empty line after each log section
pub fn ui_for_logs<'a>(
	logs: impl IntoIterator<Item = &'a Log>,
	max_width: u16,
	stage: Option<Stage>,
	show_steps: bool,
) -> Vec<Line<'static>> {
	let mut all_lines: Vec<Line> = Vec::new();
	for log in logs {
		// -- Show or not step
		if !show_steps && matches!(log.kind, Some(LogKind::RunStep)) {
			continue;
		}

		// -- Show or not stage
		// if stage in arg, but no log stage, then skip
		if stage.is_some() && stage != log.stage {
			continue;
		}

		// Render log lines
		let log_lines = comp::ui_for_log(log, max_width);
		all_lines.extend(log_lines);
		all_lines.push(Line::default()); // empty line (for now)
	}

	all_lines
}

/// Return the lines for a single log entity
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
