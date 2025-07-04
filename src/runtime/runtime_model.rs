use crate::Result;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::store::Id;
use crate::store::rt_model::{RunBmc, RunForCreate};
use crate::support::time::now_unix_time_us;

impl Runtime {
	pub async fn run_start(&self, agent_name: &str, agent_path: &str) -> Result<Id> {
		let hub = get_hub();

		let id = RunBmc::create(
			self.mm(),
			RunForCreate {
				agent_name: Some(agent_name.to_string()),
				agent_path: Some(agent_path.to_string()),
				start: Some(now_unix_time_us().into()),
			},
		)?;

		// -- For legacy terminal
		hub.publish(format!(
			"\n======= RUNNING: {agent_name}\n     Agent path: {agent_path}",
		))
		.await;

		Ok(id)
	}
}
