use crate::Result;
use crate::exec::assets;
use crate::exec::cli::CreateGitignoreArgs;
use crate::hub::get_hub;
use simple_fs::SPath;
use std::fs;

/// Executes the create-gitignore command which creates a .gitignore file from template
pub async fn exec_create_gitignore(args: CreateGitignoreArgs) -> Result<()> {
	let hub = get_hub();
	let gitignore_path = SPath::from(".gitignore");

	if gitignore_path.exists() && !args.force {
		hub.publish("-> .gitignore already exists. Skipping creation. (use --force to overwrite)")
			.await;
		return Ok(());
	}

	let zfile = assets::extract_template_zfile("template.gitignore")?;

	let existed = gitignore_path.exists();
	fs::write(&gitignore_path, zfile.content)?;

	if existed {
		hub.publish("-> Overwrote .gitignore from template (--force).").await;
	} else {
		hub.publish("-> Created .gitignore from template.").await;
	}

	Ok(())
}
