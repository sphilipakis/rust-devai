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
//! - `aip.web.parse_url(url: string | nil): table | nil`
//! - `aip.web.resolve_href(href: string | nil, base_url: string): string | nil`

use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::script::support::into_option_string;
use crate::support::{StrExt as _, W};
use crate::{Error, Result};
use mlua::{IntoLua, Lua, LuaSerdeExt, Table, Value};
use reqwest::redirect::Policy;
use reqwest::{Client, Response, header};
use std::collections::HashMap;
use url::Url;

pub fn init_module(lua: &Lua, _runtime_context: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let web_get_fn = lua.create_function(web_get)?;
	let web_post_fn = lua.create_function(web_post)?;
	let parse_url_fn = lua.create_function(web_parse_url)?;
	let resolve_href_fn = lua.create_function(web_resolve_href)?;

	table.set("get", web_get_fn)?;
	table.set("post", web_post_fn)?;
	table.set("parse_url", parse_url_fn)?;
	table.set("resolve_href", resolve_href_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Parses a URL string and returns its components as a table.
///
/// ```lua
/// -- API Signature
/// aip.web.parse_url(url: string | nil): table | nil
/// ```
///
/// ### Arguments
///
/// - `url: string | nil`: The URL string to parse. If `nil` is provided, the function returns `nil`.
///
/// ### Returns (`table | nil`)
///
/// - If the `url` is a valid string, returns a table with the following fields:
///   - `scheme: string` (e.g., "http", "https")
///   - `host: string | nil` (e.g., "example.com")
///   - `port: number | nil` (e.g., 80, 443)
///   - `path: string` (e.g., "/path/to/resource")
///   - `query: table | nil` (A Lua table where keys are query parameter names and values are their corresponding string values. E.g., `{ name = "value" }`)
///   - `fragment: string | nil` (The part of the URL after '#')
///   - `username: string` (The username for authentication, empty string if not present)
///   - `password: string | nil` (The password for authentication)
///   - `url: string` (The original or normalized URL string that was parsed)
/// - If the input `url` is `nil`, the function returns `nil`.
///
/// ### Example
///
/// ```lua
/// local parsed = aip.web.parse_url("https://user:pass@example.com:8080/path?query=val#fragment")
/// if parsed then
///   print(parsed.scheme)    -- "https"
///   print(parsed.host)      -- "example.com"
///   print(parsed.port)      -- 8080
///   print(parsed.path)      -- "/path"
///   print(parsed.query.query) -- "val"
///   print(parsed.fragment)  -- "fragment"
///   print(parsed.username)  -- "user"
///   print(parsed.password)  -- "pass"
///   print(parsed.url)       -- "https://user:pass@example.com:8080/path?query=val#fragment"
/// end
///
/// local nil_result = aip.web.parse_url(nil)
/// -- nil_result will be nil
/// ```
///
/// ### Error
///
/// Returns an error if the `url` string is provided but is invalid and cannot be parsed.
pub fn web_parse_url(lua: &Lua, url: Value) -> mlua::Result<Value> {
	let Some(url) = into_option_string(url, "aip.web.parse_url argument")? else {
		return Ok(Value::Nil);
	};

	match Url::parse(&url) {
		Ok(url) => Ok(W(url).into_lua(lua)?),
		Err(err) => Err(crate::Error::Custom(format!("Cannot parse url '{url}'. Cause: {err}")).into()),
	}
}

/// ## Lua Documentation
///
/// Resolves an `href` (like one from an HTML `<a>` tag) against a `base_url`.
///
/// ```lua
/// -- API Signature
/// aip.web.resolve_href(href: string | nil, base_url: string): string | nil
/// ```
///
/// ### Arguments
///
/// - `href: string | nil`: The href string to resolve. This can be an absolute URL, a scheme-relative URL, an absolute path, or a relative path. If `nil`, the function returns `nil`.
/// - `base_url: string`: The base URL string against which to resolve the `href`. Must be a valid absolute URL.
///
/// ### Returns (`string | nil`)
///
/// - If `href` is `nil`, returns `nil`.
/// - If `href` is already an absolute URL (e.g., "https://example.com/page"), it's returned as is.
/// - Otherwise, `href` is joined with `base_url` to form an absolute URL.
/// - Returns the resolved absolute URL string.
///
/// ### Example
///
/// ```lua
/// local base = "https://example.com/docs/path/"
///
/// -- Absolute href
/// print(aip.web.resolve_href("https://another.com/page.html", base))
/// -- Output: "https://another.com/page.html"
///
/// -- Relative path href
/// print(aip.web.resolve_href("sub/page.html", base))
/// -- Output: "https://example.com/docs/path/sub/page.html"
///
/// -- Absolute path href
/// print(aip.web.resolve_href("/other/resource.txt", base))
/// -- Output: "https://example.com/other/resource.txt"
///
/// -- Scheme-relative href
/// print(aip.web.resolve_href("//cdn.com/asset.js", base))
/// -- Output: "https://cdn.com/asset.js" (uses base_url's scheme)
///
/// print(aip.web.resolve_href("//cdn.com/asset.js", "http://example.com/"))
/// -- Output: "http://cdn.com/asset.js"
///
/// -- href is nil
/// print(aip.web.resolve_href(nil, base))
/// -- Output: nil (Lua nil)
/// ```
///
/// ### Error
///
/// Returns an error (Lua table `{ error: string }`) if:
/// - `base_url` is not a valid absolute URL.
/// - `href` and `base_url` cannot be successfully joined (e.g., due to malformed `href`).
fn web_resolve_href(lua: &Lua, (href_val, base_url_str): (Value, String)) -> mlua::Result<Value> {
	let href_opt_str = into_option_string(href_val, "aip.web.resolve_href 'href' argument")?;

	let Some(href_str) = href_opt_str else {
		return Ok(Value::Nil);
	};

	// Attempt to parse href_str as a standalone, absolute URL.
	if let Ok(parsed_href_url) = Url::parse(&href_str) {
		if !parsed_href_url.scheme().is_empty() {
			// It's already an absolute URL with a scheme.
			return Ok(Value::String(lua.create_string(&href_str)?));
		}
		// If it parsed but has no scheme (e.g. "//example.com/path", "/path", "path"),
		// it should be joined with the base_url.
	}
	// If parsing href_str failed, it's treated as a path segment to be joined with base_url.

	let base_url = Url::parse(&base_url_str).map_err(|e| {
		Error::custom(format!(
			"aip.web.resolve_href: Invalid base_url '{}'. Cause: {}",
			base_url_str, e
		))
	})?;

	match base_url.join(&href_str) {
		Ok(resolved_url) => Ok(Value::String(lua.create_string(resolved_url.as_str())?)),
		Err(e) => Err(Error::custom(format!(
			"aip.web.resolve_href: Failed to join href '{}' with base_url '{}'. Cause: {}",
			href_str, base_url_str, e
		))
		.into()),
	}
}

impl IntoLua for W<Url> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let url = self.0;
		let table = lua.create_table()?;
		table.set("scheme", url.scheme())?;
		table.set("host", url.host_str())?;
		table.set("port", url.port())?;
		table.set("path", url.path())?;
		let query = url.query_pairs().into_owned().collect::<HashMap<String, String>>();
		let query_table = if query.is_empty() {
			Value::Nil
		} else {
			lua.to_value(&query)?
		};
		table.set("query", query_table)?;
		table.set("fragment", url.fragment())?;
		table.set("username", url.username())?;
		table.set("password", url.password())?;

		table.set("url", url.as_str())?;

		Ok(Value::Table(table))
	}
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
fn web_post(lua: &Lua, (url, data): (String, Value)) -> mlua::Result<Value> {
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
	use serde_json::Value as JsonValue;
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_script_aip_web_get_simple_ok() -> Result<()> {
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
	async fn test_script_aip_web_post_json_ok() -> Result<()> {
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
	async fn test_script_aip_web_get_invalid_url() -> Result<()> {
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

	#[tokio::test]
	async fn test_script_aip_web_parse_url_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.parse_url("https://user:pass@example.com:8080/path/to/resource?key1=val1&key2=val2#fragment")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.x_get_str("scheme")?, "https");
		assert_eq!(res.x_get_str("host")?, "example.com");
		assert_eq!(res.x_get_i64("port")?, 8080);
		assert_eq!(res.x_get_str("path")?, "/path/to/resource");
		assert_eq!(res.x_get_str("/query/key1")?, "val1");
		assert_eq!(res.x_get_str("/query/key2")?, "val2");
		assert_eq!(res.x_get_str("fragment")?, "fragment");
		assert_eq!(res.x_get_str("username")?, "user");
		assert_eq!(res.x_get_str("password")?, "pass");
		assert_eq!(
			res.x_get_str("url")?,
			"https://user:pass@example.com:8080/path/to/resource?key1=val1&key2=val2#fragment"
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_parse_url_invalid() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.parse_url("not a valid url")
		"#;

		// -- Exec
		let err = match eval_lua(&lua, script) {
			Ok(_) => return Err("Should have returned an error".into()),
			Err(e) => e,
		};

		// -- Check
		let err_str = err.to_string();
		assert_contains(&err_str, "Cannot parse url 'not a valid url'");
		assert_contains(&err_str, "relative URL without a base"); // This is the specific error from url::Url::parse

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_parse_url_nil() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.parse_url(nil)
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res, JsonValue::Null, "Result should be JSON null for Lua nil");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_absolute_href() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.resolve_href("https://another.com/page.html", "https://base.com/docs/")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.as_str().ok_or("should be string")?, "https://another.com/page.html");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_relative_path_href() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.resolve_href("sub/page.html", "https://base.com/docs/")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(
			res.as_str().ok_or("should be string")?,
			"https://base.com/docs/sub/page.html"
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_absolute_path_href() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.resolve_href("/other/resource", "https://base.com/docs/path/")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(
			res.as_str().ok_or("should be string")?,
			"https://base.com/other/resource"
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_scheme_relative_href_https() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.resolve_href("//cdn.example.com/script.js", "https://base.com/docs/")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(
			res.as_str().ok_or("should be string")?,
			"https://cdn.example.com/script.js"
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_scheme_relative_href_http() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.resolve_href("//cdn.example.com/script.js", "http://base.com/docs/")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(
			res.as_str().ok_or("should be string")?,
			"http://cdn.example.com/script.js"
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_nil_href() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.resolve_href(nil, "https://base.com/docs/")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res, JsonValue::Null);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_err_invalid_base_url() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.resolve_href("path", "not-a-base-url")
		"#;

		// -- Exec
		let err = match eval_lua(&lua, script) {
			Ok(_) => return Err("Should have returned an error".into()),
			Err(e) => e,
		};

		// -- Check
		let err_str = err.to_string();
		assert_contains(&err_str, "aip.web.resolve_href: Invalid base_url 'not-a-base-url'");
		assert_contains(&err_str, "relative URL without a base");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_empty_href() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.resolve_href("", "https://base.com/docs/")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(&res, "https://base.com/docs/");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_href_is_fragment() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r##"
return aip.web.resolve_href("#section1", "https://base.com/page.html")
		"##;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(&res, "https://base.com/page.html#section1");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_href_is_query() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web")?;
		let script = r#"
return aip.web.resolve_href("?key=val", "https://base.com/page.html")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(&res, "https://base.com/page.html?key=val");

		Ok(())
	}
}

// endregion: --- Tests
