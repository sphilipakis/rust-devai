use crate::Result;
use crate::script::serde_value_to_lua_value;
use mlua::{IntoLua, Lua, Value};
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode, header};
use std::collections::HashMap;

/// Represents the result of an HTTP request made by `aip.web.get` or `aip.web.post`.
///
/// This structure is converted to a Lua table for agent scripts.
///
/// NOTE: The `content` field is a raw string by default. It is parsed into a Lua table (JSON) only if
/// `WebOptions.parse` is set to `true` and the response `content_type` is `application/json`.
///
/// ## Lua Documentation
///
/// The structure returned to Lua is:
/// ```lua
/// {
///   success: boolean,   -- True if status code is 2xx
///   status: number,     -- The HTTP status code (e.g., 200, 404)
///   url: string,        -- The final URL after redirects
///   content: string | table, -- The body of the response. If `WebOptions.parse=true` and `content_type` is JSON, it is a table (parsed JSON), otherwise a raw string.
///   content_type?: string, -- The value of the Content-Type header, if present
///   headers?: table,      -- Lua table of response headers { header_name: string | string[] }
///   error?: string       -- Contains network error, parsing error, or generic status error if not 2xx
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WebResponse {
	pub status: StatusCode,
	pub url: String,
	pub content: String,
	pub content_type: Option<String>,
	pub headers: Option<HashMap<String, Vec<String>>>,
	pub error: Option<String>, // Error originating from reqwest/network failure
	pub parse: Option<bool>,
}

// region:    --- IntoLua

impl IntoLua for WebResponse {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;

		let success = self.status.is_success();
		let status_code = self.status.as_u16() as i64;

		table.set("success", success)?;
		table.set("status", status_code)?;
		table.set("url", self.url)?;

		let content_type_str = self.content_type.as_deref().unwrap_or_default();
		let should_parse_json = self.parse.unwrap_or(false) && content_type_str.starts_with("application/json");

		let content_lua_value = if should_parse_json {
			// Attempt to parse JSON
			let json_value: serde_json::Value = serde_json::from_str(&self.content)
				.map_err(|e| crate::Error::custom(format!("Failed to parse response body as JSON: {e}")))?;

			// Convert serde_json::Value to mlua::Value using aipack logic
			serde_value_to_lua_value(lua, json_value)
				.map_err(|e| crate::Error::custom(format!("Failed to convert parsed JSON to Lua Value: {e}")))?
		} else {
			// Return content as string (current behavior)
			Value::String(lua.create_string(&self.content)?)
		};

		table.set("content", content_lua_value)?;

		if let Some(content_type) = self.content_type {
			table.set("content_type", content_type)?;
		}

		// region:    --- Headers to Lua

		if let Some(headers) = self.headers {
			let headers_tbl = lua.create_table()?;

			for (key, values) in headers {
				let value_lua = if values.len() == 1 {
					Value::String(lua.create_string(&values[0])?)
				} else {
					let list_tbl = lua.create_table()?;
					for (i, v) in values.into_iter().enumerate() {
						list_tbl.set(i + 1, v)?;
					}
					Value::Table(list_tbl)
				};
				headers_tbl.set(key, value_lua)?;
			}

			if !headers_tbl.is_empty() {
				table.set("headers", headers_tbl)?;
			}
		}

		// endregion: --- Headers to Lua

		if let Some(error) = self.error {
			table.set("error", error)?;
		} else if !success {
			// If not successful and no network/parsing error occurred, set the generic status error
			table.set("error", format!("Not a 2xx status code ({status_code})"))?;
		}

		Ok(Value::Table(table))
	}
}

// endregion: --- IntoLua

// region:    --- Constructors

impl WebResponse {
	/// Creates a new `WebResponse` from a `reqwest::Response`.
	/// Reads the full body into a string regardless of content type.
	pub async fn from_reqwest_response(response: Response, parse_response: Option<bool>) -> Result<Self> {
		let status = response.status();
		let url = response.url().to_string();

		// Get owned HeaderMap before consuming the response body
		let headers = response.headers().clone();

		let content_type = headers
			.get(header::CONTENT_TYPE)
			.and_then(|h| h.to_str().ok().map(|s| s.to_owned()));

		let content = response.text().await.map_err(crate::Error::Reqwest)?;

		let headers_map = transform_headers(headers);

		let parse = parse_response;

		Ok(Self {
			status,
			url,
			content,
			content_type,
			headers: Some(headers_map),
			error: None,
			parse,
		})
	}
}

// endregion: --- Constructors

// region:    --- Support

fn transform_headers(headers: HeaderMap) -> HashMap<String, Vec<String>> {
	headers
		.into_iter()
		.filter_map(|(name, value)| {
			// reqwest::header::HeaderMap::into_iter yields (Option<HeaderName>, HeaderValue)
			// None name means invalid header, skip
			name.map(|name| (name, value))
		})
		.fold(HashMap::<String, Vec<String>>::new(), |mut acc, (name, value)| {
			let key = name.as_str().to_lowercase();
			if let Ok(value_str) = value.to_str() {
				acc.entry(key).or_default().push(value_str.to_owned());
			}
			acc
		})
}

// endregion: --- Support
