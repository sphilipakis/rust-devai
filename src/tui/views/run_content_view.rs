use crate::store::rt_model::LogBmc;
use crate::tui::AppState;
use crate::tui::styles::CLR_BKG_GRAY_DARKER;
use crate::tui::support::RectExt;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize as _;
use ratatui::widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget as _};

pub struct RunContentView {}

impl StatefulWidget for RunContentView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Draw content
		let logs = if let Some(current_run) = state.current_run() {
			LogBmc::list_for_display(state.mm(), current_run.id)
		} else {
			Ok(Vec::new())
		};

		let mut items: Vec<ListItem> = vec![];
		match logs {
			Ok(logs) => {
				for log in logs {
					if let Some(msg) = log.message {
						// let item = Paragraph::new(message).wrap(Wrap { trim: true });
						items.push(ListItem::new(format!("{} - {} - {msg}", log.run_id, log.id)))
					} else {
						let msg = log
							.step
							.map(|s| s.to_string())
							.unwrap_or_else(|| format!("No msg or step for log id {}", log.id));
						// let item = Paragraph::new(message).wrap(Wrap { trim: true });
						items.push(ListItem::new(format!("{} - {} - {msg}", log.run_id, log.id)))
					}
				}
			}
			Err(err) => items.push(ListItem::new(format!("LogBmc::list error. {err}"))),
		}
		// let p = Paragraph::new(content).wrap(Wrap { trim: true });

		let list_w = List::new(items);

		let mut list_s = ListState::default();
		StatefulWidget::render(list_w, area.x_margin(1), buf, &mut list_s);
	}
}
