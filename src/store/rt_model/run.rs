use crate::store::base::{self, DbBmc};
use crate::store::{Id, ModelManager, Result, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Run {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub start: Option<UnixTimeUs>,
	pub end: Option<UnixTimeUs>,

	// Before All start/end
	pub ba_start: Option<UnixTimeUs>,
	pub ba_end: Option<UnixTimeUs>,

	// All tasks start/end
	pub tasks_start: Option<UnixTimeUs>,
	pub tasks_end: Option<UnixTimeUs>,

	// After All start/end
	pub aa_start: Option<UnixTimeUs>,
	pub aa_end: Option<UnixTimeUs>,

	pub agent_name: Option<String>,
	pub agent_path: Option<String>,

	pub model: Option<String>,

	pub total_cost: Option<f64>,

	pub label: Option<String>,
}

impl Run {
	pub fn is_done(&self) -> bool {
		self.end.is_some()
	}
	// pub fn has_before_all(&self) -> bool {
	// 	self.ba_start.is_some()
	// }
	// pub fn has_after_all(&self) -> bool {
	// 	self.aa_start.is_some()
	// }
}

#[derive(Debug, Clone, Fields, SqliteFromRow, Default)]
pub struct RunForCreate {
	pub start: Option<UnixTimeUs>,
	pub agent_name: Option<String>,
	pub agent_path: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct RunForUpdate {
	pub start: Option<UnixTimeUs>,
	pub end: Option<UnixTimeUs>,

	// Before All start/end
	pub ba_start: Option<UnixTimeUs>,
	pub ba_end: Option<UnixTimeUs>,

	// All tasks start/end
	pub tasks_start: Option<UnixTimeUs>,
	pub tasks_end: Option<UnixTimeUs>,

	// After All start/end
	pub aa_start: Option<UnixTimeUs>,
	pub aa_end: Option<UnixTimeUs>,

	pub agent_name: Option<String>,
	pub agent_path: Option<String>,

	pub model: Option<String>,

	pub total_cost: Option<f64>,

	pub label: Option<String>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct RunBmc;

impl DbBmc for RunBmc {
	const TABLE: &'static str = "run";
}

/// Basic CRUD
impl RunBmc {
	pub fn create(mm: &ModelManager, run_c: RunForCreate) -> Result<Id> {
		let fields = run_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, run_u: RunForUpdate) -> Result<usize> {
		let fields = run_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	#[allow(unused)]
	pub fn get(mm: &ModelManager, id: Id) -> Result<Run> {
		base::get::<Self, _>(mm, id)
	}

	pub fn list(mm: &ModelManager, list_options: Option<ListOptions>) -> Result<Vec<Run>> {
		base::list::<Self, _>(mm, list_options, None)
	}

	pub fn list_for_display(mm: &ModelManager, limit: Option<i64>) -> Result<Vec<Run>> {
		let mut options = ListOptions::from_order_bys("!id");
		if let Some(limit) = limit {
			options.limit = Some(limit);
		}
		Self::list(mm, Some(options))
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::support::time::now_unix_time_us;
	use modql::filter::OrderBy;

	#[tokio::test]
	async fn test_model_run_bmc_create() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_c = RunForCreate {
			agent_name: Some("Test Run".to_string()),
			agent_path: Some("test/path".to_string()),
			start: None,
		};

		// -- Exec
		let id = RunBmc::create(&mm, run_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);

		Ok(())
	}

	#[tokio::test]
	async fn test_model_run_bmc_update() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_c = RunForCreate {
			agent_name: Some("Test Run".to_string()),
			agent_path: Some("test/path".to_string()),
			start: None,
		};
		let id = RunBmc::create(&mm, run_c)?;

		// -- Exec
		let run_u = RunForUpdate {
			start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(&mm, id, run_u)?;

		// -- Check
		let run = RunBmc::get(&mm, id)?;
		assert!(run.start.is_some());

		Ok(())
	}

	#[tokio::test]
	async fn test_model_run_bmc_list_simple() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		for i in 0..3 {
			let run_c = RunForCreate {
				agent_name: Some(format!("label-{i}")),
				agent_path: Some(format!("path/label-{i}")),
				start: None,
			};
			RunBmc::create(&mm, run_c)?;
		}

		// -- Exec
		let runs: Vec<Run> = RunBmc::list(&mm, Some(ListOptions::default()))?;
		assert_eq!(runs.len(), 3);
		let run = runs.first().ok_or("Should have first item")?;
		assert_eq!(run.id, 1.into());
		assert_eq!(run.label, Some("label-0".to_string()));
		let run = runs.get(2).ok_or("Should have 3 items")?;
		assert_eq!(run.id, 3.into());
		assert_eq!(run.label, Some("label-2".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_run_bmc_list_from_seed() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		mm.mock_rt_seed().await?;

		// -- Exec
		let runs: Vec<Run> = RunBmc::list(&mm, Some(ListOptions::default()))?;
		assert_eq!(runs.len(), 10);
		let run = runs.first().ok_or("Should have first item")?;
		assert_eq!(run.id, 1.into());
		assert_eq!(run.label, Some("label-0".to_string()));
		let run = runs.get(2).ok_or("Should have third item")?;
		assert_eq!(run.id, 3.into());
		assert_eq!(run.label, Some("label-2".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_run_bmc_list_order_by() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		for i in 0..3 {
			let run_c = RunForCreate {
				agent_name: Some(format!("label-{i}")),
				agent_path: Some(format!("path/label-{i}")),
				start: None,
			};
			RunBmc::create(&mm, run_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let runs: Vec<Run> = RunBmc::list(&mm, Some(list_options))?;
		assert_eq!(runs.len(), 3);
		let run = runs.first().ok_or("Should have first item")?;
		assert_eq!(run.id, 3.into());
		assert_eq!(run.label, Some("label-2".to_string()));
		let run = runs.get(2).ok_or("Should have third item")?;
		assert_eq!(run.id, 1.into());
		assert_eq!(run.label, Some("label-0".to_string()));

		Ok(())
	}
}

// endregion: --- Tests
