use crate::Result;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::store::Id;
use crate::store::rt_model::{RunBmc, RunForCreate, RunForUpdate};
use crate::support::time::now_unix_time_us;

/// All the function that "record" the progress of a Runtime execution
impl Runtime {
	pub async fn rec_start(&self, agent_name: &str, agent_path: &str) -> Result<Id> {
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

	pub async fn rec_ba_start(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			ba_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	pub async fn rec_ba_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			ba_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the start of Tasks execution.
	pub async fn rec_tasks_start(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			tasks_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the end of Tasks execution.
	pub async fn rec_tasks_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			tasks_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the start of After All execution.
	pub async fn rec_aa_start(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			aa_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the end of After All execution.
	pub async fn rec_aa_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			aa_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the run as completed.
	pub async fn rec_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}
}
