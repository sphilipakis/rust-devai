use crate::script::LuaValueExt;
use mlua::{FromLua, Lua, Value};
use reqwest::ClientBuilder;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::redirect::Policy;
use std::collections::HashMap;

pub const DEFAULT_REDIRECT_LIMIT: i32 = 5;
pub const DEFAULT_UA_AIPACK: &str = "aipack";
pub const DEFAULT_UA_BROWSER: &str =
	"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Web request options used by `aip.web` functions.
///
/// All fields are optional; defaults are applied when `nil` is provided.
///
/// NOTE: By default the Web Response content will be string,
///       except when WebOptions `.parse` is true, and the content_type is `application/json`
#[derive(Default)]
pub struct WebOptions {
	/// Will be set as user agent.
	/// If `true`, defaults to `DEFAULT_UA_AIPACK`. If `false`, no UA is set. If string, sets as-is.
	/// Overrides headers `User-Agent` if present.
	pub user_agent: Option<String>,

	/// Headers to be set. Most of time single value, of vec of one
	pub headers: Option<HashMap<String, Vec<String>>>,

	/// Number of possible redirects
	/// will use .redirect(Policy::limited(n))
	pub redirect_limit: Option<i32>,

	/// If true, attempts to parse response content (e.g., JSON) based on Content-Type header.
	/// If set, the `content` field in `WebResponse` will be the parsed Lua value, otherwise a string.
	/// since: 0.8.6
	pub parse: Option<bool>,
}

impl FromLua for WebOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(WebOptions::default()),
			Value::Table(table) => {
				// -- Extract user_agent
				let user_agent = match table.get::<Value>("user_agent")? {
					Value::String(s) => Some(s.to_string_lossy()),
					Value::Boolean(true) => Some(DEFAULT_UA_AIPACK.to_owned()),
					Value::Boolean(false) => Some("".to_owned()),
					_ => None,
				};

				// -- Extract redirect_limit
				let redirect_limit = table.x_get_i64("redirect_limit").map(|v| v as i32);

				// -- Extract parse
				let parse = table.x_get_bool("parse");

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
					parse,
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

		let mut user_agent_to_set: Option<String> = self.user_agent.take();

		// If user_agent field was present, remove 'User-Agent' from headers map
		if user_agent_to_set.is_some()
			&& let Some(headers) = self.headers.as_mut()
			&& let Some(ua_key) = headers.keys().find(|k| k.eq_ignore_ascii_case("user-agent")).cloned()
		{
			headers.remove(&ua_key);
		}

		// If not explicitly set via field, try to find it in headers, and remove if found.
		if user_agent_to_set.is_none()
			&& let Some(headers) = self.headers.as_mut()
			&& let Some(ua_key) = headers.keys().find(|k| k.eq_ignore_ascii_case("user-agent")).cloned()
			&& let Some(ua_values) = headers.remove(&ua_key)
			&& let Some(first_value) = ua_values.into_iter().next()
		{
			// We must consume from headers here so it doesn't get processed later as a default header
			user_agent_to_set = Some(first_value);
		}

		// Apply default if still missing
		if user_agent_to_set.is_none() {
			user_agent_to_set = Some(DEFAULT_UA_AIPACK.to_owned());
		}

		// Set user agent in client builder if determined and not empty string
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
