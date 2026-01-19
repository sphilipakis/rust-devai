use crate::model::base::{self, DbBmc};
use crate::model::{EpochUs, Id, ModelManager, Result, ScalarEnum};
use macro_rules_attribute as mra;
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Prompt {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: EpochUs,
	pub mtime: EpochUs,

	pub kind: Option<PromptKind>,

	pub run_id: Option<Id>,
	pub task_id: Option<Id>,

	pub title: Option<String>,
	pub message: Option<String>,
	pub fields: Option<String>,  // json
	pub actions: Option<String>, // json
}

#[mra::derive(Debug, ScalarEnum!)]
pub enum PromptKind {
	Sys,
	Agent,
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct PromptForCreate {
	pub kind: Option<PromptKind>,

	pub run_id: Option<Id>,
	pub task_id: Option<Id>,

	pub title: Option<String>,
	pub message: Option<String>,
	pub fields: Option<String>,  // json
	pub actions: Option<String>, // json
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct PromptForUpdate {
	pub title: Option<String>,
	pub message: Option<String>,
	pub fields: Option<String>,  // json
	pub actions: Option<String>, // json
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct PromptFilter {
	pub run_id: Option<Id>,
	pub task_id: Option<Id>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct PromptBmc;

impl DbBmc for PromptBmc {
	const TABLE: &'static str = "prompt";
}

impl PromptBmc {
	#[allow(unused)]
	pub fn create(mm: &ModelManager, prompt_c: PromptForCreate) -> Result<Id> {
		let fields = prompt_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, prompt_u: PromptForUpdate) -> Result<usize> {
		let fields = prompt_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	#[allow(unused)]
	pub fn get(mm: &ModelManager, id: Id) -> Result<Prompt> {
		base::get::<Self, _>(mm, id)
	}

	#[allow(unused)]
	pub fn list(
		mm: &ModelManager,
		list_options: Option<ListOptions>,
		filter: Option<PromptFilter>,
	) -> Result<Vec<Prompt>> {
		let filter_fields = filter.map(|f| f.sqlite_not_none_fields());
		base::list::<Self, _>(mm, list_options, filter_fields)
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::model::{RunBmc, RunForCreate, TaskBmc, TaskForCreate};
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

	async fn create_task(mm: &ModelManager, run_id: Id, num: i64) -> Result<Id> {
		let task_c = TaskForCreate {
			run_id,
			idx: num,
			label: Some(format!("task-{num}")),
			input_content: None,
		};
		Ok(TaskBmc::create(mm, task_c)?)
	}
	// endregion: --- Support

	#[tokio::test]
	async fn test_model_prompt_bmc_create() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let task_id = create_task(&mm, run_id, 1).await?;

		// -- Exec
		let prompt_c = PromptForCreate {
			run_id: Some(run_id),
			task_id: Some(task_id),
			kind: Some(PromptKind::Agent),
			title: Some("Title 1".to_string()),
			message: Some("Message 1".to_string()),
			fields: Some("fields json".to_string()),
			actions: Some("actions json".to_string()),
		};
		let id = PromptBmc::create(&mm, prompt_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);
		let prompt: Prompt = PromptBmc::get(&mm, id)?;
		assert_eq!(prompt.kind, Some(PromptKind::Agent));
		assert_eq!(prompt.title.as_deref(), Some("Title 1"));
		assert_eq!(prompt.run_id, Some(run_id));
		assert_eq!(prompt.task_id, Some(task_id));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_prompt_bmc_update() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let prompt_c = PromptForCreate {
			run_id: Some(run_id),
			task_id: None,
			kind: None,
			title: Some("Before".to_string()),
			message: None,
			fields: None,
			actions: None,
		};
		let id = PromptBmc::create(&mm, prompt_c)?;

		// -- Exec
		let prompt_u = PromptForUpdate {
			title: Some(format!("Updated at {}", now_micro())),
			..Default::default()
		};
		PromptBmc::update(&mm, id, prompt_u)?;

		// -- Check
		let prompt = PromptBmc::get(&mm, id)?;
		assert!(prompt.title.unwrap().starts_with("Updated"));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_prompt_bmc_list_simple() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let prompt_c = PromptForCreate {
				run_id: Some(run_id),
				task_id: None,
				kind: None,
				title: Some(format!("title-{i}")),
				message: None,
				fields: None,
				actions: None,
			};
			PromptBmc::create(&mm, prompt_c)?;
		}

		// -- Exec
		let prompts: Vec<Prompt> = PromptBmc::list(&mm, Some(ListOptions::default()), None)?;

		// -- Check
		assert_eq!(prompts.len(), 3);
		let prompt = prompts.first().ok_or("Should have first item")?;
		assert_eq!(prompt.id, 1.into());
		assert_eq!(prompt.title, Some("title-0".to_string()));
		assert!(prompt.kind.is_none());

		Ok(())
	}

	#[tokio::test]
	async fn test_model_prompt_bmc_list_order_by() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let prompt_c = PromptForCreate {
				run_id: Some(run_id),
				task_id: None,
				kind: if i == 2 { Some(PromptKind::Sys) } else { None },
				title: Some(format!("title-{i}")),
				message: None,
				fields: None,
				actions: None,
			};
			PromptBmc::create(&mm, prompt_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let prompts: Vec<Prompt> = PromptBmc::list(&mm, Some(list_options), None)?;

		// -- Check
		assert_eq!(prompts.len(), 3);
		let prompt = prompts.first().ok_or("Should have first item")?;
		assert_eq!(prompt.id, 3.into());
		assert_eq!(prompt.title, Some("title-2".to_string()));
		assert_eq!(prompt.kind, Some(PromptKind::Sys));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_prompt_bmc_list_with_filter() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_1_id = create_run(&mm, "run-1").await?;
		let run_2_id = create_run(&mm, "run-2").await?;
		for run_id in [run_1_id, run_2_id] {
			for i in 0..3 {
				let prompt_c = PromptForCreate {
					run_id: Some(run_id),
					task_id: None,
					kind: if i == 2 { Some(PromptKind::Sys) } else { None },
					title: Some(format!("title-{i}")),
					message: None,
					fields: None,
					actions: None,
				};
				PromptBmc::create(&mm, prompt_c)?;
			}
		}

		// -- Exec
		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);
		let filter = PromptFilter {
			run_id: Some(run_1_id),
			..Default::default()
		};
		let prompts: Vec<Prompt> = PromptBmc::list(&mm, Some(list_options), Some(filter))?;

		// -- Check
		assert_eq!(prompts.len(), 3);
		let prompt = prompts.first().ok_or("Should have first item")?;
		assert_eq!(prompt.id, 3.into());
		assert_eq!(prompt.title, Some("title-2".to_string()));
		assert_eq!(prompt.kind, Some(PromptKind::Sys));

		Ok(())
	}
}

// endregion: --- Tests
