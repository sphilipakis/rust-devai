use crate::Result;
use crate::script::serde_value_to_lua_value;
use mlua::{IntoLua, Lua, Value};
use reqwest::{Response, StatusCode, header};

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
///   error?: string       -- Contains network error, parsing error, or generic status error if not 2xx
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WebResponse {
	pub status: StatusCode,
	pub url: String,
	pub content: String,
	pub content_type: Option<String>,
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

		let content_type = response
			.headers()
			.get(header::CONTENT_TYPE)
			.and_then(|h| h.to_str().ok().map(|s| s.to_owned()));

		let content = response.text().await.map_err(crate::Error::Reqwest)?;

		let parse = parse_response;

		Ok(Self {
			status,
			url,
			content,
			content_type,
			error: None,
			parse,
		})
	}
}

// endregion: --- Constructors
