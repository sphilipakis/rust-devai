use crate::store::Result;
use crate::store::db::Db;

#[derive(Debug, Clone)]
pub struct ModelManager {
	db: Db,
}

/// Constructors
impl ModelManager {
	pub async fn new() -> Result<Self> {
		let db = Db::new()?;
		db.recreate()?;
		Ok(Self { db })
	}
}

/// Getters
impl ModelManager {
	pub fn db(&self) -> &Db {
		&self.db
	}
}

// region:    --- Once

use crate::support::time::now_unix_time_us;
use tokio::sync::OnceCell;

#[derive(Clone, Copy)]
pub struct OnceModelManager;

impl OnceModelManager {
	/// Returns a reference to the singleton `ModelManager`, creating it on first call.
	pub async fn get(&self) -> Result<ModelManager> {
		static INSTANCE: OnceCell<ModelManager> = OnceCell::const_new();
		let val = INSTANCE.get_or_try_init(|| async { ModelManager::new().await }).await?;
		Ok(val.clone())
	}
}

// endregion: --- Once

// region:    --- Mock Seed

impl ModelManager {
	pub async fn mock_rt_seed(&self) -> Result<()> {
		use crate::store::rt_model::{RunBmc, RunForCreate};

		for i in 0..10 {
			let run_c = RunForCreate {
				agent_name: Some(format!("agent_name-{i}")),
				agent_path: Some(format!("agent_path-{i}")),
				start: Some(now_unix_time_us().into()),
			};
			let id = RunBmc::create(self, run_c)?;

			if i < 3 {
				RunBmc::start(self, id)?;
				RunBmc::end(self, id)?;
			}
		}

		Ok(())
	}
}

// endregion: --- Mock Seed
