use crate::Result;
use crate::runtime::Runtime;
use crate::store::Id;
use crate::store::rt_model::{RunBmc, RunForCreate};

impl Runtime {
	pub async fn run_start(&self, agent_path: &str) -> Result<Id> {
		let id = RunBmc::create(
			self.mm(),
			RunForCreate {
				label: Some(agent_path.to_string()),
			},
		)?;

		Ok(id)
	}
}
