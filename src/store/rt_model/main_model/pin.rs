// region:    --- Modules

use crate::store::base::{self, DbBmc};
use crate::store::{Id, ModelManager, Result, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// endregion: --- Modules

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Pin {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub run_id: Id,
	pub task_id: Option<Id>,

	pub iden: Option<String>,
	pub priority: Option<f64>,
	pub content: Option<String>,
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct PinForCreate {
	pub run_id: Id,
	pub task_id: Option<Id>,

	pub iden: Option<String>,
	pub priority: Option<f64>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct PinForUpdate {
	pub iden: Option<String>,
	pub priority: Option<f64>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct PinFilter {
	pub run_id: Option<Id>,
	pub task_id: Option<Id>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct PinBmc;

impl DbBmc for PinBmc {
	const TABLE: &'static str = "pin";
}

#[allow(unused)]
impl PinBmc {
	pub fn create(mm: &ModelManager, pin_c: PinForCreate) -> Result<Id> {
		let fields = pin_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	pub fn update(mm: &ModelManager, id: Id, pin_u: PinForUpdate) -> Result<usize> {
		let fields = pin_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	pub fn get(mm: &ModelManager, id: Id) -> Result<Pin> {
		base::get::<Self, _>(mm, id)
	}

	pub fn list(mm: &ModelManager, list_options: Option<ListOptions>, filter: Option<PinFilter>) -> Result<Vec<Pin>> {
		let filter_fields = filter.map(|f| f.sqlite_not_none_fields());
		base::list::<Self, _>(mm, list_options, filter_fields)
	}

	// Convenience helpers
	pub fn list_for_run(mm: &ModelManager, run_id: Id) -> Result<Vec<Pin>> {
		let filter = PinFilter {
			run_id: Some(run_id),
			..Default::default()
		};
		Self::list(mm, None, Some(filter))
	}

	pub fn list_for_task(mm: &ModelManager, task_id: Id) -> Result<Vec<Pin>> {
		let filter = PinFilter {
			task_id: Some(task_id),
			..Default::default()
		};
		Self::list(mm, None, Some(filter))
	}

	// --- Extras (needed for Lua `aip.pin` API)

	/// Fetch a pin by its `uid` (convenience wrapper).
	pub fn get_by_uid(mm: &ModelManager, uid: Uuid) -> Result<Pin> {
		base::get_by_uid::<Self, _>(mm, uid)
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::store::rt_model::{RunBmc, RunForCreate, TaskBmc, TaskForCreate};
	use crate::support::time::now_micro;
	use modql::filter::OrderBy;

	// region:    --- Support

	async fn create_run(mm: &ModelManager, label: &str) -> Result<Id> {
		let run_c = RunForCreate {
			parent_id: None,
			agent_name: Some(label.to_string()),
			agent_path: Some(format!("path/{label}")),
			has_task_stages: None,
			has_prompt_parts: None,
		};
		Ok(RunBmc::create(mm, run_c)?)
	}

	async fn create_task(mm: &ModelManager, run_id: Id, idx: i64) -> Result<Id> {
		let task_c = TaskForCreate {
			run_id,
			idx,
			label: Some(format!("task-{idx}")),
			input_content: None,
		};
		Ok(TaskBmc::create(mm, task_c)?)
	}

	// endregion: --- Support

	#[tokio::test]
	async fn test_model_pin_bmc_create() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let task_id = create_task(&mm, run_id, 1).await?;

		// -- Exec
		let pin_c = PinForCreate {
			run_id,
			task_id: Some(task_id),
			iden: Some("work-summary".to_string()),
			priority: Some(0.5),
			content: Some(r#"{"type":"Marker","content":"First Pin"}"#.to_string()),
		};
		let id = PinBmc::create(&mm, pin_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);
		let pin: Pin = PinBmc::get(&mm, id)?;
		assert_eq!(pin.task_id, Some(task_id));
		assert_eq!(pin.priority, Some(0.5));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_pin_bmc_update() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let pin_c = PinForCreate {
			run_id,
			task_id: None,
			iden: Some("pin-1".to_string()),
			priority: None,
			content: Some("Old content".to_string()),
		};
		let id = PinBmc::create(&mm, pin_c)?;

		// -- Exec
		let pin_u = PinForUpdate {
			content: Some(format!("Updated at {}", now_micro())),
			priority: Some(1.0),
			..Default::default()
		};
		PinBmc::update(&mm, id, pin_u)?;

		// -- Check
		let pin = PinBmc::get(&mm, id)?;
		assert!(pin.content.unwrap().starts_with("Updated"));
		assert_eq!(pin.priority, Some(1.0));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_pin_bmc_list_simple() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let pin_c = PinForCreate {
				run_id,
				task_id: None,
				iden: Some(format!("pin-{i}")),
				priority: Some(i as f64),
				content: Some(format!("content-{i}")),
			};
			PinBmc::create(&mm, pin_c)?;
		}

		// -- Exec
		let pins: Vec<Pin> = PinBmc::list(&mm, Some(ListOptions::default()), None)?;

		// -- Check
		assert_eq!(pins.len(), 3);
		let pin = pins.first().ok_or("Should have first item")?;
		assert_eq!(pin.id, 1.into());
		assert_eq!(pin.iden, Some("pin-0".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_pin_bmc_list_order_by() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let pin_c = PinForCreate {
				run_id,
				task_id: None,
				iden: Some(format!("pin-{i}")),
				priority: Some(i as f64),
				content: Some(format!("content-{i}")),
			};
			PinBmc::create(&mm, pin_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let pins: Vec<Pin> = PinBmc::list(&mm, Some(list_options), None)?;

		// -- Check
		assert_eq!(pins.len(), 3);
		let pin = pins.first().ok_or("Should have first item")?;
		assert_eq!(pin.id, 3.into());
		assert_eq!(pin.iden, Some("pin-2".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_pin_bmc_list_with_filter() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_1_id = create_run(&mm, "run-1").await?;
		let run_2_id = create_run(&mm, "run-2").await?;
		for run_id in [run_1_id, run_2_id] {
			for i in 0..3 {
				let pin_c = PinForCreate {
					run_id,
					task_id: None,
					iden: Some(format!("pin-{i}")),
					priority: None,
					content: Some(format!("content-{i}")),
				};
				PinBmc::create(&mm, pin_c)?;
			}
		}

		// -- Exec
		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);
		let filter = PinFilter {
			run_id: Some(run_1_id),
			..Default::default()
		};
		let pins: Vec<Pin> = PinBmc::list(&mm, Some(list_options), Some(filter))?;

		// -- Check
		assert_eq!(pins.len(), 3);
		let pin = pins.first().ok_or("Should have first item")?;
		assert_eq!(pin.id, 3.into());
		assert_eq!(pin.run_id, run_1_id);

		Ok(())
	}
}

// endregion: --- Tests
