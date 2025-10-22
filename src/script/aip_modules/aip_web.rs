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
//! - `aip.web.get(url: string, options?: WebOptions): WebResponse`
//! - `aip.web.post(url: string, data: string | table, options?: WebOptions): WebResponse`
//! - `aip.web.parse_url(url: string | nil): table | nil`
//! - `aip.web.resolve_href(href: string | nil, base_url: string): string | nil`
//!
//! ### Constants
//!
//! - `aip.web.UA_BROWSER: string`: Default browser User Agent string.
//! - `aip.web.UA_AIPACK: string`: Default aipack User Agent string (`aipack`).
//!
//! ### Related Types
//!
//! Where `WebOptions` is:
//! ```lua
//! {
//!   user_agent?: string | boolean,   
//!   headers?: table,                  -- { header_name: string | string[] }
//!   redirect_limit?: number,          -- number of redirects to follow (default 5)
//!   parse?: boolean                   -- If true, attempts to parse JSON response content (Content-Type: application/json). Content defaults to string otherwise.
//! }
//!
//! - user_agent
//!  - If boolean 'true', sets 'aipack'.
//!  - If boolean 'false', prevents setting UA.
//!  - If string, sets as-is, can use `aip.web.UA_BROWSER` for a constant of a commong browser user-agent
//!  - If undefined, will default to `aipack` or what is in the `.headers``
//! ```

use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::script::support::into_option_string;
use crate::support::W;
use crate::types::{DEFAULT_UA_AIPACK, DEFAULT_UA_BROWSER, WebOptions, WebResponse};
use crate::{Error, Result};
use mlua::{FromLua as _, IntoLua, Lua, LuaSerdeExt, Table, Value};
use reqwest::{Client, header};
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

	table.set("UA_AIPACK", DEFAULT_UA_AIPACK)?;
	table.set("UA_BROWSER", DEFAULT_UA_BROWSER)?;

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
///   - `page_url: string` (The url without the query and fragment
/// - If the input `url` is `nil`, the function returns `nil`.
///
/// ### Example
///
/// ```lua
/// local parsed = aip.web.parse_url("https://user:pass@example.com:8080/path/to/page.html?param1=val#fragment")
/// if parsed then
///   print(parsed.scheme)       -- "https"
///   print(parsed.host)         -- "example.com"
///   print(parsed.port)         -- 8080
///   print(parsed.path)         -- "/path/to/page.html"
///   print(parsed.query.param1) -- "val"
///   print(parsed.fragment)     -- "fragment"
///   print(parsed.username)     -- "user"
///   print(parsed.password)     -- "pass"
///   print(parsed.url)          -- "https://user:pass@example.com:8080/path/to/page.html?query=val#fragment"
///   print(parsed.page_url)     -- "https://user:pass@example.com:8080/path/to/page.html"
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
	if let Ok(parsed_href_url) = Url::parse(&href_str)
		&& !parsed_href_url.scheme().is_empty()
	{
		// It's already an absolute URL with a scheme.
		return Ok(Value::String(lua.create_string(&href_str)?));
	}
	// If parsing href_str failed, it's treated as a path segment to be joined with base_url.

	let base_url = Url::parse(&base_url_str).map_err(|e| {
		Error::custom(format!(
			"aip.web.resolve_href: Invalid base_url '{base_url_str}'. Cause: {e}"
		))
	})?;

	match base_url.join(&href_str) {
		Ok(resolved_url) => Ok(Value::String(lua.create_string(resolved_url.as_str())?)),
		Err(e) => Err(Error::custom(format!(
			"aip.web.resolve_href: Failed to join href '{href_str}' with base_url '{base_url_str}'. Cause: {e}"
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

		// page_url
		let mut page_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
		if let Some(port) = url.port() {
			page_url.push(':');
			page_url.push_str(&format!(":{port}"));
		}
		page_url.push_str(url.path());
		table.set("page_url", page_url)?;

		Ok(Value::Table(table))
	}
}

/// ## Lua Documentation
///
/// Makes an HTTP GET request to the specified URL.
///
/// ```lua
/// -- API Signature
/// aip.web.get(url: string, options?: WebOptions): WebResponse
/// ```
///
/// ### Arguments
///
/// - `url: string`: The URL to make the GET request to.
/// - `options?: WebOptions`: Optional web request options (user_agent, headers, redirect_limit)
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
///   content: string | table, // The body of the response. Defaults to string, but can be a table (parsed JSON) if `WebOptions.parse` is true and `Content-Type` is `application/json`.
///   content_type?: string, // The value of the Content-Type header, if present
///   error?: string,   // Contains network error, parsing error, or generic status error if not 2xx
/// }
/// ```
///
/// ### Example
///
/// ```lua
/// local response = aip.web.get("https://google.com")
/// print(response.status) -- 200
/// print(response.content) -- HTML content of Google's homepage
///
/// -- With options
/// local response = aip.web.get("https://api.example.com", {
///   user_agent = "true",
///   headers = { ["Authorization"] = "Bearer token123" },
///   redirect_limit = 10
/// })
/// ```
///
/// ### Error
///
/// Returns an error if the web request cannot be made (e.g., invalid URL, network error).  Does not throw an error for non-2xx status codes. Check the `success` field in the `WebResponse`.
fn web_get(lua: &Lua, (url, opts): (String, Option<Value>)) -> mlua::Result<Value> {
	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;
	let res: mlua::Result<Value> = tokio::task::block_in_place(|| {
		rt.block_on(async {
			let mut builder = Client::builder();

			let opts_val = opts.unwrap_or(Value::Nil);
			let web_opts = WebOptions::from_lua(opts_val, lua)?;
			let parse_response = web_opts.parse;
			builder = web_opts.apply_to_reqwest_builder(builder);

			let client = builder.build().map_err(crate::Error::from)?;

			let res: mlua::Result<Value> = match client.get(&url).send().await {
				Ok(response) => {
					let web_res = WebResponse::from_reqwest_response(response, parse_response).await?;
					Ok(web_res.into_lua(lua)?)
				}
				Err(err) => Err(crate::Error::custom(format!(
					"\
Fail to do aip.web.get for url: {url}
Cause: {err}"
				))
				.into()),
			};

			if res.is_ok() {
				get_hub().publish_sync(format!("-> lua web::get OK ({url}) "));
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
/// aip.web.post(url: string, data: string | table, options?: WebOptions): WebResponse
/// ```
///
/// ### Arguments
///
/// - `url: string`: The URL to make the POST request to.
/// - `data: string | table`: The data to send in the request body.  If a string is provided, the `Content-Type` header will be set to `plain/text`. If a table is provided, the `Content-Type` header will be set to `application/json` and the table will be serialized as JSON.
/// - `options?: WebOptions`: Optional web request options (user_agent, headers, redirect_limit)
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
///   content: string | table, // The body of the response. Defaults to string, but can be a table (parsed JSON) if `WebOptions.parse` is true and `Content-Type` is `application/json`.
///   content_type?: string, // The value of the Content-Type Header, if present
///   error?: string,   // Contains network error, parsing error, or generic status error if not 2xx
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
///
/// -- POST with options
/// local response = aip.web.post("https://api.example.com", { data = "value" }, {
///   user_agent = "MyApp/1.0",
///   headers = { ["X-API-Key"] = "secret123" }
/// })
/// ```
///
/// ### Error
///
/// Returns an error if the web request cannot be made (e.g., invalid URL, network error, data serialization error). Does not throw an error for non-2xx status codes. Check the `success` field in the `WebResponse`.
fn web_post(lua: &Lua, (url, data, opts): (String, Value, Option<Value>)) -> mlua::Result<Value> {
	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;
	let res: mlua::Result<Value> = tokio::task::block_in_place(|| {
		rt.block_on(async {
			let mut builder = Client::builder();

			let opts_val = opts.unwrap_or(Value::Nil);
			let web_opts = WebOptions::from_lua(opts_val, lua)?;
			let parse_response = web_opts.parse;
			builder = web_opts.apply_to_reqwest_builder(builder);

			let client = builder.build().map_err(crate::Error::from)?;

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
				Ok(response) => {
					let web_res = WebResponse::from_reqwest_response(response, parse_response).await?;
					Ok(web_res.into_lua(lua)?)
				}
				Err(err) => Err(crate::Error::custom(format!(
					"\
Fail to do aip.web.post for url: {url}
Cause: {err}"
				))
				.into()),
			};

			if res.is_ok() {
				get_hub().publish_sync(format!("-> lua web::post OK ({url}) "));
			}

			// return the Result<Dynamic, Error>
			res
		})
	});

	res
}

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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
		let script = r#"
local url = "https://postman-echo.com/post"
local res = aip.web.post(url, {some = "stuff"}, {parse = true})
return res
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
		let script = r#"
return aip.web.parse_url(nil)
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res, JsonValue::Null, "Result should be JSON null for Lua nil");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_script_aip_web_get_with_headers_capture() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web").await?;
		let script = r#"
local url = "https://postman-echo.com/response-headers?Content-Type=text/plain&Set-Cookie=session1=a;%20HttpOnly&Set-Cookie=session2=b&X-Custom=val_single"
local res = aip.web.get(url)
return res
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		// Check standard header that should be there (case insensitive match for extraction)
		let date_header = res.x_get_str("/headers/content-type")?;
		assert!(!date_header.is_empty(), "text/plain; charset=utf-8");

		// Check custom single value header
		let custom_header = res.x_get_str("/headers/set-cookie")?;
		assert_eq!(custom_header, "session1=a; HttpOnly");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_web_resolve_href_ok_absolute_href() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
		let lua = setup_lua(aip_web::init_module, "web").await?;
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
