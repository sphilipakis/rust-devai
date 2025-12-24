use crate::model::{LogBmc, WorkBmc};
use crate::tui::AppState;
use crate::tui::style;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget as _};

use crate::tui::core::AppStage;
use ratatui::widgets::{Block, BorderType, Clear, Padding};

pub struct InstallView;

impl StatefulWidget for InstallView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		match state.stage() {
			AppStage::Installing => render_installing(area, buf, state),
			AppStage::Installed => render_installed(area, buf, state),
			_ => (),
		}
	}
}

fn render_installing(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	let pack_ref = state.installing_pack_ref().unwrap_or("Unknown pack");
	let work_id = state.current_work_id();

	let mut detail_msg = String::new();
	if let Some(work_id) = work_id {
		if let Ok(work) = WorkBmc::get(state.mm(), work_id) {
			if let Some(msg) = work.message {
				detail_msg = msg;
			}
		}
	}

	let mut log_msg = String::new();
	if let Ok(logs) = LogBmc::list_for_run_only(state.mm(), 0.into()) {
		if let Some(last_log) = logs.last() {
			if let Some(msg) = &last_log.message {
				log_msg = msg.clone();
			}
		}
	}

	let mut lines = vec![
		Line::from("Installing pack ...")
			.alignment(Alignment::Center)
			.style(style::STL_POPUP_TITLE),
		Line::default(),
		Line::from(pack_ref).alignment(Alignment::Center).style(style::STL_FIELD_VAL),
		Line::default(),
	];

	// Detail or Log message
	if !detail_msg.is_empty() {
		lines.push(Line::from(detail_msg).alignment(Alignment::Center).style(style::STL_TXT));
	} else if !log_msg.is_empty() {
		lines.push(Line::from(log_msg).alignment(Alignment::Center).style(style::CLR_TXT_800));
	}

	render_dialog_base(area, buf, lines);
}

fn render_installed(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	let work_id = state.current_work_id();
	let mut pack_info = "Unknown pack".to_string();

	if let Some(work_id) = work_id {
		if let Ok(work) = WorkBmc::get(state.mm(), work_id) {
			if let Some(msg) = work.message {
				pack_info = msg;
			}
		}
	}

	let lines = vec![
		Line::from("✔ Installed")
			.alignment(Alignment::Center)
			.style(style::CLR_TXT_DONE),
		Line::default(),
		Line::from(pack_info).alignment(Alignment::Center).style(style::STL_FIELD_VAL),
	];

	render_dialog_base(area, buf, lines);
}

// region:    --- Support

fn render_dialog_base(area: Rect, buf: &mut Buffer, lines: Vec<Line>) {
	// Dialog layout
	let dialog_width = 60;
	let dialog_height = 8;

	let [_, mid_v, _] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Fill(1),
			Constraint::Length(dialog_height),
			Constraint::Length(4), // Push it slightly higher
		])
		.areas(area);

	let [_, content_a, _] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Fill(1),
			Constraint::Length(dialog_width),
			Constraint::Fill(1),
		])
		.areas(mid_v);

	// Clear and Background
	Clear.render(content_a, buf);

	Paragraph::new(lines)
		.block(
			Block::bordered()
				.border_type(BorderType::Rounded)
				.border_style(style::CLR_TXT_WHITE)
				.bg(style::CLR_BKG_BLACK)
				.padding(Padding::new(2, 2, 1, 1)),
		)
		.render(content_a, buf);
}

// endregion: --- Support
