use crate::script::LuaValueExt;
use mlua::{FromLua, Lua, Value};
use reqwest::ClientBuilder;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::redirect::Policy;
use std::collections::HashMap;

const DEFAULT_REDIRECT_LIMIT: i32 = 5;
const DEFAULT_WHEN_TRUE_USER_AGENT: &str =
	"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Web request options used by `aip.web` functions.
///
/// All fields are optional; defaults are applied when `nil` is provided.
#[derive(Default)]
pub struct WebOptions {
	/// Will be set as user agent.
	/// If "true" then will set a good browser default
	/// if Text value, then will be set as is.
	/// Will override headers `User-Agent` if present.
	pub user_agent: Option<String>,

	/// Headers to be set. Most of time single value, of vec of one
	pub headers: Option<HashMap<String, Vec<String>>>,

	/// Number of possible redirects
	/// will use .redirect(Policy::limited(n))
	pub redirect_limit: Option<i32>,
}

impl FromLua for WebOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(WebOptions::default()),
			Value::Table(table) => {
				// -- Extract user_agent
				let user_agent = match table.get::<Value>("user_agent")? {
					Value::String(s) => {
						let s_owned = s.to_string_lossy();
						if s_owned.eq_ignore_ascii_case("true") {
							Some(DEFAULT_WHEN_TRUE_USER_AGENT.to_owned())
						} else {
							Some(s_owned)
						}
					}
					Value::Boolean(true) => Some(DEFAULT_WHEN_TRUE_USER_AGENT.to_owned()),
					Value::Boolean(false) => Some("".to_owned()),
					_ => None,
				};

				// -- Extract redirect_limit
				let redirect_limit = table.x_get_i64("redirect_limit").map(|v| v as i32);

				// -- Extract headers
				let headers = if let Ok(headers_table) = table.get::<mlua::Table>("headers") {
					let mut headers_map = HashMap::new();
					for pair in headers_table.pairs::<String, Value>() {
						let (key, value) = pair?;
						match value {
							Value::String(s) => {
								headers_map.insert(key, vec![s.to_string_lossy()]);
							}
							Value::Table(t) => {
								let mut values = Vec::new();
								for v in t.sequence_values::<String>() {
									values.push(v?);
								}
								headers_map.insert(key, values);
							}
							_ => {
								return Err(mlua::Error::FromLuaConversionError {
									from: value.type_name(),
									to: "String or Array".to_string(),
									message: Some("Header values must be strings or arrays of strings".into()),
								});
							}
						}
					}
					Some(headers_map)
				} else {
					None
				};

				Ok(WebOptions {
					user_agent,
					headers,
					redirect_limit,
				})
			}
			other => Err(mlua::Error::FromLuaConversionError {
				from: other.type_name(),
				to: "WebOptions".to_string(),
				message: Some("Expected nil or a table for WebOptions".into()),
			}),
		}
	}
}

impl WebOptions {
	/// Apply web options to a reqwest ClientBuilder.
	/// Consumes self and returns the modified builder.
	pub fn apply_to_reqwest_builder(mut self, mut client_builder: ClientBuilder) -> ClientBuilder {
		// Apply redirect limit
		let limit = self.redirect_limit.unwrap_or(DEFAULT_REDIRECT_LIMIT);
		client_builder = client_builder.redirect(Policy::limited(limit as usize));

		// region:    --- Extract & Set user_agent

		let mut user_agent_to_set: Option<String> = None;
		let mut skip_ua_header_processing = false;

		// 1. Handle explicit user_agent field (overrides headers)
		if let Some(ua_opt) = self.user_agent.take() {
			skip_ua_header_processing = true;
			// If ua_opt is "", it means user explicitly set user_agent=false and wants no user agent.
			if !ua_opt.is_empty() {
				user_agent_to_set = Some(ua_opt);
			}
		}

		// 2. Handle user_agent from headers if not explicitly set via field
		if !skip_ua_header_processing
			&& let Some(headers) = self.headers.as_mut()
			&& let Some(ua_key) = headers.keys().find(|k| k.eq_ignore_ascii_case("user-agent")).cloned()
			&& let Some(ua_values) = headers.remove(&ua_key)
			&& let Some(first_value) = ua_values.into_iter().next()
		{
			user_agent_to_set = Some(first_value);
		}

		// 3. Default user agent if none was determined and we are not skipping (i.e., user_agent: false)
		if user_agent_to_set.is_none() && !skip_ua_header_processing {
			user_agent_to_set = Some("aipack".to_owned());
		}

		// 4. Set user agent in client builder
		if let Some(ua) = user_agent_to_set
			&& !ua.is_empty()
		{
			client_builder = client_builder.user_agent(ua);
		}

		// endregion: --- Extract & Set user_agent

		// region:    --- Extract & set header

		// Apply other headers (excluding UA handled above or explicitly skipped)
		if let Some(headers) = self.headers {
			let mut header_map = HeaderMap::new();

			for (key, values) in headers {
				// If the explicit user_agent field was used (skip_ua_header_processing = true),
				// we must ignore any 'User-Agent' key remaining in the headers map.
				if key.eq_ignore_ascii_case("user-agent") {
					continue;
				}

				if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
					for value in values {
						if let Ok(header_value) = HeaderValue::from_str(&value) {
							header_map.append(header_name.clone(), header_value);
						}
					}
				}
			}

			if !header_map.is_empty() {
				client_builder = client_builder.default_headers(header_map);
			}
		}

		// endregion: --- Extract & set header

		client_builder
	}
}
