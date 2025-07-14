use crate::store::rt_model::{Run, Task};
use crate::store::{EndState, RunningState, UnixTimeUs};
use crate::tui::styles;
use ratatui::text::Span;

pub fn el_running_ico(arg: impl Into<RunningState>) -> Span<'static> {
	let arg = arg.into();

	match arg {
		RunningState::Waiting => Span::styled("⏸", styles::CLR_TXT_WAITING),
		RunningState::Running => Span::styled("▶", styles::CLR_TXT_RUNNING),
		RunningState::Ended(end_state) => match end_state {
			Some(EndState::Ok) => Span::styled("✔", styles::CLR_TXT_DONE),
			Some(EndState::Cancel) => Span::styled("✗", styles::CLR_TXT),
			Some(EndState::Skip) => Span::styled("⏭", styles::CLR_TXT),
			Some(EndState::Err) => Span::styled("✘", styles::CLR_TXT_RED),
			None => Span::styled("?", styles::CLR_TXT),
		},
	}
}
