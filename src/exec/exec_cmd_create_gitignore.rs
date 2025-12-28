use crate::Result;
use crate::exec::assets;
use crate::exec::cli::CreateGitignoreArgs;
use crate::hub::get_hub;
use simple_fs::SPath;
use std::fs;

/// Executes the create-gitignore command which creates a .gitignore file from template
pub async fn exec_create_gitignore(_args: CreateGitignoreArgs) -> Result<()> {
	let hub = get_hub();
	let gitignore_path = SPath::from(".gitignore");

	if gitignore_path.exists() {
		hub.publish("-> .gitignore already exists. Skipping creation.").await;
		return Ok(());
	}

	let zfile = assets::extract_template_zfile("template.gitignore")?;

	fs::write(&gitignore_path, zfile.content)?;

	hub.publish("-> Created .gitignore from template.").await;

	Ok(())
}
