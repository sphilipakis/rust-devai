#![allow(unused)]
use crate::dir_context::DirContext;
use crate::exec::cli::NewArgs;
use crate::support::AsStrsExt;
use crate::term::{init_term, prompt_input, prompt_select, safer_println};
use crate::{Result, term};

// (name, title)
const AGENT_TEMPLATES: [(&str, &str); 2] = [
	//
	("hello-world", "Hello World Agent"),
	("ask", "Generic Ask Agent with parametric prompt"),
];

/// exec for the New command
/// Will create a new pack in base or workspace custom (not sure yet)
///
/// NOTE: THIS IS DISABLED FOR NOW
pub async fn exec_new(new_args: NewArgs, _dir_context: DirContext) -> Result<()> {
	// let aipack_paths = dir_context.aipack_paths();

	let mut stdout = init_term()?;

	// -- Prompt the template
	let max_len = AGENT_TEMPLATES.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
	let labels = AGENT_TEMPLATES
		.iter()
		.map(|(name, title)| format!(" {name:>width$} - {title}", width = max_len))
		.collect::<Vec<_>>();
	let template_idx = prompt_select(&mut stdout, "Select the agent template you want", &labels.x_as_strs())?;

	// -- Prompt the name
	let name_prompt = if let Some(name) = new_args.agent_path {
		format!("Agent path/name ({name}):")
	} else {
		"Agent path/name:".to_string()
	};
	let agent_name = prompt_input(&mut stdout, &name_prompt, false)?;
	let agent_name = agent_name.trim();

	let agent_name = if agent_name.is_empty() { "my-agent" } else { agent_name };

	let template_name = AGENT_TEMPLATES
		.get(template_idx)
		.map(|(name, _)| name)
		.ok_or_else(|| format!("No template found for index {template_idx}"))?;

	let confirm_prompt = format!(
		r#"
Creating new agent
    name: {agent_name}
template: {template_name}
"#
	);
	safer_println(&confirm_prompt, true);

	let confirm = prompt_input(&mut stdout, "Confirm creation (Y/n)", false)?;

	// TODO: NOT IMPLEMENTED YET
	if term::is_input_yes(&confirm) {
		safer_println("Will do the create", true);
	} else {
		safer_println("Agent creation cancelled", true);
	}

	Ok(())
}
