use crate::Result;
use crate::store::rt_model::{RunBmc, RunForCreate, TaskBmc, TaskForCreate};
use crate::store::{Id, ModelManager};

pub fn create_run(mm: &ModelManager, label: &str) -> Result<Id> {
	let run_c = RunForCreate {
		parent_id: None,
		agent_name: Some(label.to_string()),
		agent_path: Some(format!("path/{label}")),
		has_task_stages: None,
		has_prompt_parts: None,
	};
	Ok(RunBmc::create(mm, run_c)?)
}

pub fn create_task(mm: &ModelManager, run_id: Id, idx: i64) -> Result<Id> {
	let task_c = TaskForCreate {
		run_id,
		idx,
		label: Some(format!("task-{idx}")),
		input_content: None,
	};
	Ok(TaskBmc::create(mm, task_c)?)
}
