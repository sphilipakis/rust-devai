use crate::store::{EndState, RunningState};
use crate::tui::style;
use ratatui::text::Span;

pub fn el_running_ico(arg: impl Into<RunningState>) -> Span<'static> {
	let arg = arg.into();

	match arg {
		RunningState::NotScheduled => Span::styled("·", style::CLR_TXT_WAITING),
		RunningState::Waiting => Span::styled("⏸", style::CLR_TXT_WAITING),
		RunningState::Running => Span::styled("▶", style::CLR_TXT_RUNNING),
		RunningState::Ended(end_state) => match end_state {
			Some(EndState::Ok) => Span::styled("✔", style::CLR_TXT_DONE),
			Some(EndState::Cancel) => Span::styled("■", style::CLR_TXT),
			Some(EndState::Skip) => Span::styled("■", style::CLR_TXT_BLUE),
			Some(EndState::Err) => Span::styled("✘", style::CLR_TXT_RED),
			None => Span::styled("?", style::CLR_TXT),
		},
	}
}

pub fn ico_scroll_up() -> Span<'static> {
	Span::styled("▲", style::CLR_TXT_700)
}

pub fn ico_scroll_down() -> Span<'static> {
	Span::styled("▼", style::CLR_TXT_700)
}
