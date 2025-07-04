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

// region:    --- Mock Seed

impl ModelManager {
	pub async fn mock_rt_seed(&self) -> Result<()> {
		use crate::store::rt_model::{RunBmc, RunForCreate};

		for i in 0..10 {
			let run_c = RunForCreate {
				label: Some(format!("run-{i}")),
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
