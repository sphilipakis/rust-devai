use crate::Error;
use crate::exec::ExecActionEvent;
use crate::hub::get_hub;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, terminal};
use std::io::Write as _;
use tokio::sync::mpsc;

pub fn safer_println(msg: &str, interactive: bool) {
	if interactive {
		let stdout = std::io::stdout();
		let mut stdout_lock = stdout.lock(); // Locking stdout to avoid multiple open handles

		for line in msg.split("\n") {
			// Clear the current line and move the cursor to the start
			execute!(
				stdout_lock,
				terminal::Clear(ClearType::CurrentLine),
				cursor::MoveToColumn(0)
			)
			.expect("Failed to clear line and reset cursor");
			// Write the line content
			// write!(stdout_lock, "{}", line).expect("Failed to write to stdout");
			println!("{line}");
			stdout_lock.flush().expect("Failed to flush stdout");
		}
		// Flush to ensure everything is displayed properly
	} else {
		println!("{msg}");
	}
}

pub async fn send_to_executor(exec_tx: &mpsc::Sender<ExecActionEvent>, exec_cmd: ExecActionEvent) {
	// clear_last_n_lines(1);
	if let Err(err) = exec_tx.send(exec_cmd).await {
		get_hub()
			.publish(Error::cc("start_app - cannot send ExecCommand::Redo", err))
			.await;
	};
}
