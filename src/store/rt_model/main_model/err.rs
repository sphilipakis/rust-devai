use crate::store::base::{self, DbBmc};
use crate::store::{ContentTyp, Id, ModelManager, Result, Stage, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct ErrRec {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	pub stage: Option<Stage>,

	// Foreign keys (optional – allow global errors)
	pub run_id: Option<Id>,
	pub task_id: Option<Id>,

	pub typ: Option<String>,
	pub content: Option<String>,
}

/// Same fields as the main table but without the IDs/ctime/mtime.
/// Note: `typ` uses `ContentTyp` for stronger typing on create.
#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct ErrForCreate {
	pub stage: Option<Stage>,

	pub run_id: Option<Id>,
	pub task_id: Option<Id>,

	pub typ: Option<ContentTyp>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct ErrForUpdate {
	pub typ: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct ErrFilter {
	pub run_id: Option<Id>,
	pub task_id: Option<Id>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct ErrBmc;

impl DbBmc for ErrBmc {
	const TABLE: &'static str = "err";
}

impl ErrBmc {
	pub fn create(mm: &ModelManager, err_c: ErrForCreate) -> Result<Id> {
		let fields = err_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, err_u: ErrForUpdate) -> Result<usize> {
		let fields = err_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	pub fn get(mm: &ModelManager, id: Id) -> Result<ErrRec> {
		base::get::<Self, _>(mm, id)
	}

	#[allow(unused)]
	pub fn list(
		mm: &ModelManager,
		list_options: Option<ListOptions>,
		filter: Option<ErrFilter>,
	) -> Result<Vec<ErrRec>> {
		let filter_fields = filter.map(|f| f.sqlite_not_none_fields());
		base::list::<Self, _>(mm, list_options, filter_fields)
	}

	#[allow(unused)]
	pub fn list_for_run(mm: &ModelManager, run_id: Id) -> Result<Vec<ErrRec>> {
		let filter = ErrFilter {
			run_id: Some(run_id),
			..Default::default()
		};
		Self::list(mm, None, Some(filter))
	}

	#[allow(unused)]
	pub fn list_for_task(mm: &ModelManager, task_id: Id) -> Result<Vec<ErrRec>> {
		let filter = ErrFilter {
			task_id: Some(task_id),
			..Default::default()
		};
		Self::list(mm, None, Some(filter))
	}
}

// endregion: --- Bmc

// region:    --- Tests

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
	async fn test_model_err_bmc_create() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let task_id = create_task(&mm, run_id, 1).await?;

		// -- Exec
		let err_c = ErrForCreate {
			stage: None,
			run_id: Some(run_id),
			task_id: Some(task_id),
			typ: Some(ContentTyp::Text),
			content: Some("Something went wrong".to_string()),
		};
		let id = ErrBmc::create(&mm, err_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);
		let err_rec = ErrBmc::get(&mm, id)?;
		assert_eq!(err_rec.run_id, Some(run_id));
		assert_eq!(err_rec.task_id, Some(task_id));
		assert_eq!(err_rec.typ, Some("Text".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_err_bmc_update() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let err_c = ErrForCreate {
			stage: None,
			run_id: Some(run_id),
			task_id: None,
			typ: Some(ContentTyp::Text),
			content: Some("Before update".to_string()),
		};
		let id = ErrBmc::create(&mm, err_c)?;

		// -- Exec
		let err_u = ErrForUpdate {
			content: Some(format!("Updated at {}", now_micro())),
			..Default::default()
		};
		ErrBmc::update(&mm, id, err_u)?;

		// -- Check
		let err_rec = ErrBmc::get(&mm, id)?;
		assert!(err_rec.content.unwrap().starts_with("Updated"));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_err_bmc_list_simple() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		for i in 0..3 {
			let err_c = ErrForCreate {
				stage: None,
				run_id: None,
				task_id: None,
				typ: Some(ContentTyp::Text),
				content: Some(format!("err-{i}")),
			};
			ErrBmc::create(&mm, err_c)?;
		}

		// -- Exec
		let errs: Vec<ErrRec> = ErrBmc::list(&mm, Some(ListOptions::default()), None)?;

		// -- Check
		assert_eq!(errs.len(), 3);
		let err_rec = errs.first().ok_or("Should have first item")?;
		assert_eq!(err_rec.id, 1.into());
		assert_eq!(err_rec.content, Some("err-0".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_err_bmc_list_order_by() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		for i in 0..3 {
			let err_c = ErrForCreate {
				stage: None,
				run_id: None,
				task_id: None,
				typ: if i == 2 {
					Some(ContentTyp::Json)
				} else {
					Some(ContentTyp::Text)
				},
				content: Some(format!("err-{i}")),
			};
			ErrBmc::create(&mm, err_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let errs: Vec<ErrRec> = ErrBmc::list(&mm, Some(list_options), None)?;

		// -- Check
		assert_eq!(errs.len(), 3);
		let err_rec = errs.first().ok_or("Should have first item")?;
		assert_eq!(err_rec.id, 3.into());
		assert_eq!(err_rec.content, Some("err-2".to_string()));
		assert_eq!(err_rec.typ, Some("Json".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_err_bmc_list_with_filter() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_1_id = create_run(&mm, "run-1").await?;
		let run_2_id = create_run(&mm, "run-2").await?;
		for run_id in [run_1_id, run_2_id] {
			for i in 0..3 {
				let err_c = ErrForCreate {
					stage: None,
					run_id: Some(run_id),
					task_id: None,
					typ: Some(ContentTyp::Text),
					content: Some(format!("err-{i}")),
				};
				ErrBmc::create(&mm, err_c)?;
			}
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);
		let filter = ErrFilter {
			run_id: Some(run_1_id),
			..Default::default()
		};

		// -- Exec
		let errs: Vec<ErrRec> = ErrBmc::list(&mm, Some(list_options), Some(filter))?;

		// -- Check
		assert_eq!(errs.len(), 3);
		let err_rec = errs.first().ok_or("Should have first item")?;
		assert_eq!(err_rec.run_id, Some(run_1_id));

		Ok(())
	}
}

// endregion: --- Tests

// endregion: --- Tests
