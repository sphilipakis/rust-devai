//! Defines the `web` module, used in the lua engine
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `web` module exposes functions to make HTTP requests.
//!
//! ### Functions
//!
//! - `aip.web.get(url: string): WebResponse`
//! - `aip.web.post(url: string, data: string | table): WebResponse`

use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::support::StrExt as _;
use crate::{Error, Result};
use mlua::{Lua, LuaSerdeExt, Table, Value};
use reqwest::redirect::Policy;
use reqwest::{Client, Response, header};

pub fn init_module(lua: &Lua, _runtime_context: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let web_get_fn = lua.create_function(move |lua, (url,): (String,)| web_get(lua, url))?;
	let web_post_fn = lua.create_function(move |lua, (url, data): (String, Value)| web_post(lua, url, data))?;

	table.set("get", web_get_fn)?;
	table.set("post", web_post_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Makes an HTTP GET request to the specified URL.
///
/// ```lua
/// -- API Signature
/// aip.web.get(url: string): WebResponse
/// ```
///
/// ### Arguments
///
/// - `url: string`: The URL to make the GET request to.
///
/// ### Returns (WebResponse)
///
/// Returns a table containing the response information.
///
/// ```ts
/// {
///   success: boolean, // Indicates if the request was successful (status code 2xx)
///   status: number,   // The HTTP status code of the response
///   url: string,      // The URL that was requested
///   content: string | table    // The content of the response. If the response content type is `application/json`, the `content` field will be a Lua table. Otherwise, it will be a string.
/// }
/// ```
///
/// ### Example
///
/// ```lua
/// local response = aip.web.get("https://google.com")
/// print(response.status) -- 200
/// print(response.content) -- HTML content of Google's homepage
/// ```
///
/// ### Error
///
/// Returns an error if the web request cannot be made (e.g., invalid URL, network error).  Does not throw an error for non-2xx status codes. Check the `success` field in the `WebResponse`.
fn web_get(lua: &Lua, url: String) -> mlua::Result<Value> {
	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;
	let res: mlua::Result<Value> = tokio::task::block_in_place(|| {
		rt.block_on(async {
			let client = Client::builder()
				.redirect(Policy::limited(5)) // Set to follow up to 5 redirects
				.build()
				.map_err(crate::Error::from)?;

			let res: mlua::Result<Value> = match client.get(&url).send().await {
				Ok(response) => get_lua_response_value(lua, response, &url).await,
				Err(err) => Err(crate::Error::custom(format!(
					"\
Fail to do aip.web.get for url: {url}
Cause: {err}"
				))
				.into()),
			};

			if res.is_ok() {
				get_hub().publish_sync(format!("-> lua web::get OK ({}) ", url));
			}

			// return the Result<Dynamic, Error>
			res
		})
	});

	res
}

/// ## Lua Documentation
///
/// Makes an HTTP POST request to the specified URL with the given data.
///
/// ```lua
/// -- API Signature
/// aip.web.post(url: string, data: string | table): WebResponse
/// ```
///
/// ### Arguments
///
/// - `url: string`: The URL to make the POST request to.
/// - `data: string | table`: The data to send in the request body.  If a string is provided, the `Content-Type` header will be set to `plain/text`. If a table is provided, the `Content-Type` header will be set to `application/json` and the table will be serialized as JSON.
///
/// ### Returns (WebResponse)
///
/// Returns a table containing the response information.
///
/// ```ts
/// {
///   success: boolean, // Indicates if the request was successful (status code 2xx)
///   status: number,   // The HTTP status code of the response
///   url: string,      // The URL that was requested
///   content: string | table    // The content of the response. If the response content type is `application/json`, the `content` field will be a Lua table. Otherwise, it will be a string.
/// }
/// ```
///
/// ### Example
///
/// ```lua
/// -- POST with plain text
/// local response = aip.web.post("https://example.com/api", "plain text data")
///
/// -- POST with JSON data
/// local response = aip.web.post("https://example.com/api", { key1 = "value1", key2 = "value2" })
/// ```
///
/// ### Error
///
/// Returns an error if the web request cannot be made (e.g., invalid URL, network error, data serialization error). Does not throw an error for non-2xx status codes. Check the `success` field in the `WebResponse`.
fn web_post(lua: &Lua, url: String, data: Value) -> mlua::Result<Value> {
	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;
	let res: mlua::Result<Value> = tokio::task::block_in_place(|| {
		rt.block_on(async {
			let client = Client::builder()
				.redirect(Policy::limited(5)) // Set to follow up to 5 redirects
				.build()
				.map_err(crate::Error::from)?;

			let mut request_builder = client.post(&url);

			// Set Content-Type and body based on the type of 'data'
			match data {
				Value::String(s) => {
					request_builder = request_builder
						.header(header::CONTENT_TYPE, "plain/text")
						.body(s.to_string_lossy());
				}
				Value::Table(table) => {
					let json: serde_json::Value = serde_json::to_value(table).map_err(|err| {
						crate::Error::custom(format!(
							"Cannot searlize to json the argument given to the post.\n    Cause: {err}"
						))
					})?;
					// mlua provides the serialize features.
					request_builder = request_builder
						.header(header::CONTENT_TYPE, "application/json")
						.body(json.to_string());
				}
				_ => {
					return Err(mlua::Error::RuntimeError(
						"Data must be a string or a table".to_string(),
					));
				}
			}

			let res: mlua::Result<Value> = match request_builder.send().await {
				Ok(response) => get_lua_response_value(lua, response, &url).await,
				Err(err) => Err(crate::Error::custom(format!(
					"\
Fail to do aip.web.post for url: {url}
Cause: {err}"
				))
				.into()),
			};

			if res.is_ok() {
				get_hub().publish_sync(format!("-> lua web::post OK ({}) ", url));
			}

			// return the Result<Dynamic, Error>
			res
		})
	});

	res
}

// region:    --- Support

async fn get_lua_response_value(lua: &Lua, response: Response, url: &str) -> mlua::Result<Value> {
	let content_type = get_content_type(&response);
	//
	let status = response.status();
	let success = status.is_success();
	let status_code = status.as_u16() as i64;

	if success {
		// TODO: needs to reformat this error to match the lua function
		let res = lua.create_table()?;
		res.set("success", true)?;
		res.set("status", status_code)?;
		res.set("url", url)?;
		let content = response.text().await.map_err(Error::Reqwest)?;
		let content = get_content_value_for_content_type(lua, content_type, &content)?;
		res.set("content", content)?;
		Ok(Value::Table(res))
	} else {
		let res = lua.create_table()?;
		res.set("success", false)?;
		res.set("status", status_code)?;
		res.set("url", url)?;
		let content = response.text().await.unwrap_or_default();
		let content = Value::String(lua.create_string(&content)?);

		res.set("content", content)?;
		res.set("error", format!("Not a 2xx status code ({status_code})"))?;
		// NOTE: This is not an error, as the web request was sent
		Ok(Value::Table(res))
	}
}

/// Returns the appropriate lua Value type depending of the content type.
/// - If `application/json` it will be a Value::Table
/// - If anything else (for now), will be Value::String
fn get_content_value_for_content_type(lua: &Lua, content_type: Option<String>, content: &str) -> Result<Value> {
	let content: Value = if content_type.x_contains("application/json") {
		// parse content as json
		let content: serde_json::Value = serde_json::from_str(content)
			.map_err(|err| crate::Error::custom(format!("Fail to parse web response as json.\n    Cause: {err}")))?;

		lua.to_value(&content)?
	} else {
		Value::String(lua.create_string(content)?)
	};
	Ok(content)
}

fn get_content_type(response: &Response) -> Option<String> {
	response
		.headers()
		.get(header::CONTENT_TYPE)
		.map(|h| h.to_str().unwrap_or_default().to_lowercase())
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules::aip_web;
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_web_get_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
local url = "https://phet-dev.colorado.edu/html/build-an-atom/0.0.0-3/simple-text-only-test-page.html"
return aip.web.get(url)
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let content = res.x_get_str("content")?;
		assert_contains(content, "This page tests that simple text can be");
		assert_eq!(res.x_get_i64("status")?, 200, "status code");
		assert!(res.x_get_bool("success")?, "success should be true");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_web_post_json_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
local url = "https://postman-echo.com/post"
return aip.web.post(url, {some = "stuff"})
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let content = res.pointer("/content").ok_or("Should have content")?;
		assert_eq!(content.x_get_str("/json/some")?, "stuff");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_web_get_invalid_url() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
local url = "https://this-cannot-go/anywhere-or-can-it.aip"
return aip.web.get(url)
		"#;

		// -- Exec
		let err = match eval_lua(&lua, script) {
			Ok(_) => return Err("Should have returned an error".into()),
			Err(e) => e,
		};

		// -- Check
		let err_str = err.to_string();
		assert_contains(&err_str, "Fail to do aip.web.get");
		assert_contains(&err_str, "https://this-cannot-go/anywhere-or-can-it.aip");

		Ok(())
	}
}

// endregion: --- Tests
