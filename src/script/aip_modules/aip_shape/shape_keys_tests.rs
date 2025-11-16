type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use crate::_test_support::{assert_contains, eval_lua, setup_lua};
use crate::script::aip_modules::aip_shape::init_module;
use serde_json::json;

#[tokio::test]
async fn test_lua_aip_shape_keys_select_simple() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 1, name = "Alice", email = "a@x.com", role = "admin" }
			local keys = { "id", "email" }
			return aip.shape.select_keys(rec, keys)
		"#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"id": 1,
		"email": "a@x.com"
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_keys_select_missing_ignored() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 2, name = "Bob" }
			local keys = { "id", "missing" }
			return aip.shape.select_keys(rec, keys)
		"#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"id": 2
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_keys_select_rec_unchanged() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 3, name = "Cara", email = "c@x.com" }
			local keys = { "name" }
			local sel = aip.shape.select_keys(rec, keys)
			return { sel = sel, rec = rec }
		"#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"sel": { "name": "Cara" },
		"rec": { "id": 3, "name": "Cara", "email": "c@x.com" }
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_keys_select_invalid_key_type_err() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 1, name = "Alice" }
			local keys = { "id", 123 }
			local ok, err = pcall(function()
				return aip.shape.select_keys(rec, keys)
			end)
			if ok then
				return "should not reach"
			else
				return err
			end
		"#;

	// -- Exec
	let res = eval_lua(&lua, script);

	// -- Check
	let Err(err) = res else {
		panic!("Expected error, got {res:?}");
	};
	let err_str = err.to_string();
	assert_contains(&err_str, "aip.shape.select_keys - Key names must be strings");

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_keys_extract_simple() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 1, name = "Alice", email = "a@x.com", role = "admin" }
			local keys = { "id", "email" }
			local extracted = aip.shape.extract_keys(rec, keys)
			return { extracted = extracted, rec = rec }
		"#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"extracted": { "id": 1, "email": "a@x.com" },
		"rec": { "name": "Alice", "role": "admin" }
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_keys_extract_missing_ignored() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 2, name = "Bob" }
			local keys = { "id", "missing" }
			local extracted = aip.shape.extract_keys(rec, keys)
			return { extracted = extracted, rec = rec }
		"#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"extracted": { "id": 2 },
		"rec": { "name": "Bob" }
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_keys_extract_invalid_key_type_err() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 1, name = "Alice" }
			local keys = { "id", 123 }
			local ok, err = pcall(function()
				return aip.shape.extract_keys(rec, keys)
			end)
			if ok then
				return "should not reach"
			else
				return err
			end
		"#;

	// -- Exec
	let res = eval_lua(&lua, script);

	// -- Check
	let Err(err) = res else {
		panic!("Expected error, got {res:?}");
	};
	let err_str = err.to_string();
	assert_contains(&err_str, "aip.shape.extract_keys - Key names must be strings");

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_keys_remove_simple() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 1, name = "Alice", email = "a@x.com" }
			local n = aip.shape.remove_keys(rec, { "email" })
			return { n = n, rec = rec }
		"#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"n": 1,
		"rec": { "id": 1, "name": "Alice" }
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_keys_remove_missing_ignored() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 2, name = "Bob" }
			local n = aip.shape.remove_keys(rec, { "email", "id" })
			return { n = n, rec = rec }
		"#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"n": 1,
		"rec": { "name": "Bob" }
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_keys_remove_invalid_key_type_err() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local rec = { id = 1, name = "Alice" }
			local keys = { "id", 123 }
			local ok, err = pcall(function()
				return aip.shape.remove_keys(rec, keys)
			end)
			if ok then
				return "should not reach"
			else
				return err
			end
		"#;

	// -- Exec
	let res = eval_lua(&lua, script);

	// -- Check
	let Err(err) = res else {
		panic!("Expected error, got {res:?}");
	};
	let err_str = err.to_string();
	assert_contains(&err_str, "aip.shape.remove_keys - Key names must be strings");

	Ok(())
}
