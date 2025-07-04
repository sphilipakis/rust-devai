use crate::Result;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::store::Id;
use crate::store::rt_model::{RunBmc, RunForCreate};

impl Runtime {
	pub async fn run_start(&self, agent_name: &str, agent_path: &str) -> Result<Id> {
		let hub = get_hub();

		let id = RunBmc::create(
			self.mm(),
			RunForCreate {
				label: Some(agent_path.to_string()),
			},
		)?;

		// -- For legacy terminal
		hub.publish(format!(
			"\n======= RUNNING: {agent_name}\n     Agent path: {agent_path}\n",
		))
		.await;

		Ok(id)
	}
}
