//! Runtime rec method for lua calls

use crate::Result;
use crate::runtime::Runtime;
use crate::model::{LogBmc, LogKind, RuntimeCtx};

impl Runtime {
	pub fn rec_log_with_rt_ctx(&self, rt_ctx: &RuntimeCtx, log_kind: LogKind, msg: &str) -> Result<()> {
		LogBmc::create_log_with_rt_ctx(self.mm(), rt_ctx, log_kind, msg)?;

		Ok(())
	}
}
