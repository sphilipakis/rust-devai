use crate::Result;
use crate::support::event::{OneShotRx, OneShotTx, oneshot};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

// region:    --- Types

#[derive(Debug)]
pub struct PromptParams {
	pub message: String,
	pub one_shot_res: OneShotTx<String>,
}

impl PromptParams {
	pub fn new(message: impl Into<String>) -> (Self, OneShotRx<String>) {
		let message = message.into();
		let (tx, rx) = oneshot::<String>();
		(
			Self {
				message,
				one_shot_res: tx,
			},
			rx,
		)
	}
}

// endregion: --- Types

pub async fn prompt(param: &PromptParams) -> Result<()> {
	let PromptParams { message, one_shot_res } = param;

	let mut stdout = io::stdout();
	let mut stdin = BufReader::new(io::stdin());
	let mut input = String::new();

	stdout.write_all(message.as_bytes()).await?;
	stdout.flush().await?;

	stdin.read_line(&mut input).await?;

	one_shot_res.send_async(input).await?;

	Ok(())
}
