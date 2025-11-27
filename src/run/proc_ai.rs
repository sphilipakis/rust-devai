use crate::Result;
use crate::agent::{Agent, AgentOptions, PromptPart, parse_prompt_part_options};
use crate::hub::get_hub;
use crate::model::Id;
use crate::run::pricing::{model_pricing, price_it};
use crate::run::{AiResponse, Attachments, DryMode, RunBaseOptions};
use crate::runtime::Runtime;
use crate::support::hbs::hbs_render;
use crate::support::text::{self, format_duration, format_usage};
use genai::chat::{CacheControl, ChatMessage, ChatRequest, ChatResponse, ContentPart};
use genai::{ModelIden, ModelName};
use serde_json::Value;
use simple_fs::SPath;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Instant;

pub struct ProcAiResponse {
	pub ai_response: Option<AiResponse>,
}

pub fn build_chat_messages(
	runtime: &Runtime,
	agent: &Agent,
	before_all: &Value,
	input: &Value,
	data: &Value,
	attachments: &Attachments,
) -> Result<Vec<ChatMessage>> {
	let data_scope = HashMap::from([
		// The hbs scope data
		// Note: for now, we do not add the before all
		("data", data),
		("input", input),
		("before_all", before_all),
	]);

	let mut chat_messages: Vec<ChatMessage> = Vec::new();

	// -- Add the eventual attachments
	for att in attachments {
		// Resolve
		let file_path = SPath::new(&att.file_source);
		let file_path = match runtime.resolve_path_default(file_path, None) {
			Ok(file_path) => file_path,
			Err(err) => {
				let chat_msg = ChatMessage::user(format!(
					"Error while attaching file '{}'\nCause: {err}",
					att.file_source
				));
				chat_messages.push(chat_msg);
				continue;
			}
		};

		let chat_msg = match ContentPart::from_binary_file(&file_path) {
			Ok(file_cp) => {
				let file_name = att
					.file_name
					.as_deref()
					.unwrap_or(file_path.file_name().unwrap_or("no file name"));

				let m = format!(
					"Here is file attachment.
File Path: '{file_path}'
File Name: '{file_name}'"
				);
				let m = if let Some(desc) = &att.title {
					format!("{m}\nFile Title: {desc}")
				} else {
					m
				};
				let text = format!("{m}\n");

				ChatMessage::user(vec![
					//
					ContentPart::from_text(text),
					file_cp,
				])
			}
			Err(err) => ChatMessage::user(format!("Error while attaching file '{file_path}'\nCause: {err}")),
		};

		chat_messages.push(chat_msg);
	}

	// -- Add the prompt parts from the agent (.aip markdown template)
	for prompt_part in agent.prompt_parts() {
		let PromptPart {
			kind,
			content,
			options_str,
		} = prompt_part;

		// Note: If we have an options_str, then add it as the first line
		//       this way it can take advantage of being rendered
		//       and then, we will extract it later
		let (options_line, content) = if let Some(options_str) = options_str {
			(true, Cow::Owned(format!("{options_str}\n{content}")))
		} else {
			(false, Cow::Borrowed(content))
		};

		let rendered_content = hbs_render(content.as_str(), &data_scope)?;

		// If options_line, then we extract it
		let (options_str, rendered_content) = if options_line {
			text::extract_first_line(rendered_content)
		} else {
			(String::new(), rendered_content)
		};
		let options_str = options_str.trim();

		// For now, only add if not empty
		if !rendered_content.trim().is_empty() {
			let options = if !options_str.is_empty() {
				parse_prompt_part_options(options_str)?
			} else {
				None
			};
			let options = if options.as_ref().map(|v| v.cache).unwrap_or(false) {
				Some(CacheControl::Ephemeral.into())
			} else {
				None
			};
			chat_messages.push(ChatMessage {
				role: kind.into(),
				content: rendered_content.into(),
				options,
			})
		}
	}

	Ok(chat_messages)
}

#[allow(clippy::too_many_arguments)]
pub async fn process_ai(
	runtime: &Runtime,
	client: &genai::Client,
	run_base_options: &RunBaseOptions,
	run_model_resolved: &ModelName,
	run_id: Id,
	task_id: Id,
	agent: Agent,
	chat_messages: Vec<ChatMessage>,
) -> Result<ProcAiResponse> {
	let hub = get_hub();

	let rt_step = runtime.rt_step();
	let rt_model = runtime.rt_model();

	let is_inst_empty = chat_messages.is_empty();

	// TODO: Might want to handle if no instruction.
	if run_base_options.verbose() {
		hub.publish("\n").await;
		for msg in chat_messages.iter() {
			hub.publish(format!(
				"-- {role}:\n{content}",
				role = msg.role,
				content = msg.content.joined_texts().unwrap_or_default()
			))
			.await;
		}
	}

	// if dry_mode req, we stop
	// NOTE: dry_mode will be checked also upstream
	if matches!(run_base_options.dry_mode(), DryMode::Req) {
		return Ok(ProcAiResponse { ai_response: None });
	}

	// -- Now execute the instruction
	let model_resolved = agent.model_resolved();
	if run_model_resolved != model_resolved {
		// -- Rt Update Task - Model
		rt_model.update_task_model_ov(run_id, task_id, model_resolved).await?;
	}

	let ai_response: Option<AiResponse> = if !is_inst_empty {
		let prompt_size: usize = chat_messages.iter().map(|c| c.size()).sum();

		// Rt Step Ai Gen start
		rt_step.step_task_ai_gen_start(run_id, task_id, prompt_size as i64).await?;

		let res = process_send_to_genai(
			runtime,
			client,
			&agent,
			run_base_options,
			run_id,
			task_id,
			model_resolved,
			chat_messages,
		)
		.await;

		// Rt Step Ai Gen end
		rt_step.step_task_ai_gen_end(run_id, task_id).await?;

		let ai_response = res?;
		Some(ai_response)
	}
	// if we do not have an instruction, just return null
	else {
		hub.publish("-! No instruction, skipping genai.").await;
		None
	};

	Ok(ProcAiResponse { ai_response })
}

#[allow(clippy::too_many_arguments)]
async fn process_send_to_genai(
	runtime: &Runtime,
	client: &genai::Client,
	agent: &Agent,
	run_base_options: &RunBaseOptions,
	run_id: Id,
	task_id: Id,
	model_resolved: &ModelName,
	chat_messages: Vec<ChatMessage>,
) -> Result<AiResponse> {
	let hub = get_hub();

	let rt_model = runtime.rt_model();

	let chat_req = ChatRequest::from_messages(chat_messages);

	hub.publish(format!("-> Sending rendered instruction to {model_resolved} ..."))
		.await;

	if let Ok(service_target) = client.resolve_service_target(model_resolved).await
		&& let Some(pricing) = model_pricing(&service_target.model)
	{
		// If error, that's fine. Might want to trace it.
		let _ = rt_model.update_task_model_pricing(run_id, task_id, &pricing).await;
	}

	let start = Instant::now();
	let chat_options = agent.genai_chat_options();
	let chat_res = client.exec_chat(model_resolved, chat_req, Some(chat_options)).await?;
	let duration = start.elapsed();

	// region:    --- First Info Part

	let duration_msg = format!("Duration: {duration_str}", duration_str = format_duration(duration));
	// this is for the duration in second with 3 digit for milli (for the AI Response)
	let duration_sec = duration.as_secs_f64(); // Convert to f64
	let duration_sec = (duration_sec * 1000.0).round() / 1000.0; // Round to 3 decimal places

	let mut info = duration_msg;

	// Compute the price
	let price_usd = get_price(&chat_res);

	// -- Rt Rec - Update Cost
	if let Some(price_usd) = price_usd {
		let _ = rt_model.update_task_cost(run_id, task_id, price_usd).await;
	}

	// add to info
	if let Some(price_usd) = price_usd {
		info = format!("{info} | ~${price_usd}")
	}

	let usage_msg = format_usage(&chat_res.usage);
	info = format!("{info} | {usage_msg}");

	// endregion: --- First Info Part

	hub.publish(format!(
		"<- ai_response content received - {model_name} | {info}",
		model_name = chat_res.provider_model_iden.model_name
	))
	.await;

	let ChatResponse {
		content,
		reasoning_content,
		usage,
		model_iden: res_model_iden,
		provider_model_iden,
		..
	} = chat_res;

	// -- Rt Rec - Update Task Usage
	rt_model
		.update_task_usage(run_id, task_id, &usage, &provider_model_iden)
		.await?;

	let ai_response_content = content.into_joined_texts().filter(|s| !s.is_empty());
	let ai_response_reasoning_content = reasoning_content;

	let model_info = format_model(agent, &res_model_iden, &provider_model_iden, &agent.options());
	if run_base_options.verbose() {
		hub.publish(format!(
			"\n-- AI Output ({model_info})\n\n{content}\n",
			content = ai_response_content.as_deref().unwrap_or_default()
		))
		.await;
	}

	let info = format!("{info} | {model_info}",);

	Ok(AiResponse {
		content: ai_response_content,
		reasoning_content: ai_response_reasoning_content,
		model_name: res_model_iden.model_name,
		adapter_kind: res_model_iden.adapter_kind,
		duration_sec,
		price_usd,
		usage,
		info,
	})
}

// region:    --- Support

fn get_price(chat_res: &ChatResponse) -> Option<f64> {
	let provider = chat_res.model_iden.adapter_kind.as_lower_str();
	let model_name = &*chat_res.model_iden.model_name;
	price_it(provider, model_name, &chat_res.usage)
}

/// Model: gemini-2.0-flash | Adapter: Gemini
/// TODO: Might want to use the agent model somehow
fn format_model(
	_agent: &Agent,
	res_model_iden: &ModelIden,
	res_provider_model_iden: &ModelIden,
	agent_options: &AgentOptions,
) -> String {
	// let model_iden = agent.model_resolved();
	let model_section = if *res_model_iden.model_name != *res_provider_model_iden.model_name {
		format!(
			"Model: {model_name} ({provider_model_name}) ",
			model_name = res_model_iden.model_name,
			provider_model_name = res_provider_model_iden.model_name
		)
	} else {
		format!("Model: {model_name} ", model_name = res_model_iden.model_name)
	};

	let temp_section = if let Some(temp) = agent_options.temperature() {
		Cow::Owned(format!(" | Temperature: {temp}"))
	} else {
		Cow::Borrowed("")
	};

	let top_p_section = if let Some(top_p) = agent_options.top_p() {
		Cow::Owned(format!(" | top_p: {top_p}"))
	} else {
		Cow::Borrowed("")
	};

	format!(
		"{model_section}| Adapter: {adapter_kind}{temp_section}{top_p_section}",
		adapter_kind = res_model_iden.adapter_kind,
	)
}

// endregion: --- Support
