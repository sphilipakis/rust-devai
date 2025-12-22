use crate::model::Stage;
use crate::model::{Log, LogKind};
use crate::tui::style;
use crate::tui::view::comp;
use ratatui::style::Color;
use ratatui::text::Line;

/// NOTE: Add empty line after each log section
#[allow(unused)]
pub fn ui_for_logs<'a>(
	logs: impl IntoIterator<Item = &'a Log>,
	max_width: u16,
	stage: Option<Stage>,
	show_steps: bool,
	path_color: Option<Color>,
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
		let log_lines = comp::ui_for_log(log, max_width, path_color);
		all_lines.extend(log_lines);
		all_lines.push(Line::default()); // empty line (for now)
	}

	all_lines
}

/// Return the lines for a single log entity
pub fn ui_for_log(log: &Log, max_width: u16, path_color: Option<Color>) -> Vec<Line<'static>> {
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

	super::ui_for_marker_section_str(content, marker_txt_style, max_width, None, None, None, path_color)
}

/// Build logs UI and attach LinkZones to create section-wide hover/click for eligible logs.
pub fn ui_for_logs_with_hover<'a>(
	logs: impl IntoIterator<Item = &'a Log>,
	max_width: u16,
	stage: Option<Stage>,
	show_steps: bool,
	link_zones: &mut crate::tui::core::LinkZones,
	path_color: Option<Color>,
) -> Vec<Line<'static>> {
	use crate::tui::core::Action;

	let mut all_lines: Vec<Line<'static>> = Vec::new();

	for log in logs {
		// -- Filter by stage if provided
		if let Some(s) = stage
			&& log.stage != Some(s)
		{
			continue;
		}

		// -- Filter steps when requested
		if !show_steps && let Some(LogKind::RunStep) = log.kind {
			continue;
		}

		let Some(kind) = log.kind else {
			continue;
		};

		// Prepare the original (pre-format) content to be copied on click.
		let raw_content: String = match (log.message.as_ref(), log.kind.as_ref()) {
			(_, Some(LogKind::RunStep)) => log.step_as_str().to_string(),
			(Some(msg), _) => msg.clone(),
			_ => "No Step not MSG for log".to_string(),
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

		let is_hover_target = is_hover_log(log);
		let action = if is_hover_target {
			Some(Action::ToClipboardCopy(raw_content.clone()))
		} else {
			None
		};

		let lines = super::ui_for_marker_section_str(
			&raw_content,
			marker_txt_style,
			max_width,
			None,
			Some(link_zones),
			action,
			path_color,
		);

		all_lines.extend(lines);

		// Add empty separator line (do not attach zones to this line)
		all_lines.push(Line::default());
		link_zones.inc_current_line_by(1);
	}

	all_lines
}

pub fn is_hover_log(log: &Log) -> bool {
	match log.kind {
		Some(LogKind::AgentPrint) => true,
		Some(LogKind::AgentSkip) => true,
		Some(LogKind::SysInfo) => {
			if let Some(msg) = log.message.as_deref() {
				msg.to_ascii_lowercase().contains("ping")
			} else {
				false
			}
		}
		_ => false,
	}
}
