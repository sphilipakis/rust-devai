type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use crate::_test_support::{assert_contains, eval_lua, setup_lua};
use crate::script::aip_modules::aip_shape::init_module;
use serde_json::json;

#[tokio::test]
async fn test_lua_aip_shape_to_record_simple() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local names  = { "id", "name", "email" }
            local values = { 1, "Alice", "alice@example.com" }
            return aip.shape.to_record(names, values)
        "#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"id": 1,
		"name": "Alice",
		"email": "alice@example.com"
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_to_record_extra_values_truncated() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local names  = { "id", "name" }
            local values = { 1, "Alice", "EXTRA" }
            return aip.shape.to_record(names, values)
        "#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"id": 1,
		"name": "Alice"
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_to_record_extra_names_truncated() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local names  = { "id", "name", "email" }
            local values = { 2, "Bob" }
            return aip.shape.to_record(names, values)
        "#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"id": 2,
		"name": "Bob"
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_to_record_invalid_name_type() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local ok, err = pcall(function()
                return aip.shape.to_record({ "id", 123, "email" }, { 3, "Cara", "c@x.com" })
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
	assert_contains(&err_str, "aip.shape.to_record - Column names must be strings");

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_to_records_simple() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local names = { "id", "name" }
            local rows  = {
              { 1, "Alice" },
              { 2, "Bob"   },
            }
            return aip.shape.to_records(names, rows)
        "#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!([
		{ "id": 1, "name": "Alice" },
		{ "id": 2, "name": "Bob" }
	]);
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_to_records_rows_var_len_truncation() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local names = { "id", "name", "email" }
            local rows  = {
              { 1, "Alice" },                    -- shorter row
              { 2, "Bob", "b@x.com", "EXTRA" },  -- longer row
            }
            return aip.shape.to_records(names, rows)
        "#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!([
		{ "id": 1, "name": "Alice" },
		{ "id": 2, "name": "Bob", "email": "b@x.com" }
	]);
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_to_records_row_not_table_err() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local names = { "id", "name" }
            local rows  = {
              { 1, "Alice" },
              "INVALID_ROW"
            }
            local ok, err = pcall(function()
              return aip.shape.to_records(names, rows)
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
	assert_contains(&err_str, "aip.shape.to_records - Each row must be a table (list)");

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_to_records_invalid_name_type() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local names = { "id", 999, "email" }
            local rows  = { { 1, "Alice", "a@x.com" } }
            local ok, err = pcall(function()
              return aip.shape.to_records(names, rows)
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
	assert_contains(&err_str, "aip.shape.to_records - Column names must be strings");

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_record_to_values_auto_order() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local rec = { id = 1, name = "Alice", email = "a@x.com" }
            return aip.shape.record_to_values(rec)
        "#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!(["a@x.com", 1, "Alice"]);
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_record_to_values_with_names_nulls() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local rec = { id = 1, name = "Alice" }
            local names = { "name", "id", "missing" }
            return aip.shape.record_to_values(rec, names)
        "#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!(["Alice", 1, null]);
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_columnar_to_records_simple() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local cols = {
              id    = { 1, 2, 3 },
              name  = { "Alice", "Bob", "Cara" },
              email = { "a@x.com", "b@x.com", "c@x.com" },
            }
            return aip.shape.columnar_to_records(cols)
        "#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!([
		{ "id": 1, "name": "Alice", "email": "a@x.com" },
		{ "id": 2, "name": "Bob",   "email": "b@x.com" },
		{ "id": 3, "name": "Cara",  "email": "c@x.com" }
	]);
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_columnar_to_records_len_mismatch_err() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
            local cols = {
              id   = { 1, 2 },
              name = { "Alice" }, -- mismatch length
            }
            local ok, err = pcall(function()
              return aip.shape.columnar_to_records(cols)
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
	assert_contains(
		&err_str,
		"aip.shape.columnar_to_records - All columns must have the same length",
	);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_records_to_columnar_simple() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local recs = {
			  { id = 1, name = "Alice" },
			  { id = 2, name = "Bob" },
			}
			return aip.shape.records_to_columnar(recs)
		"#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"id":   [1, 2],
		"name": ["Alice", "Bob"]
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_records_to_columnar_intersection() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local recs = {
			  { id = 1, name = "Alice", email = "a@x.com" },
			  { id = 2, name = "Bob" }, -- missing email
			}
			return aip.shape.records_to_columnar(recs)
		"#;

	// -- Exec
	let res = eval_lua(&lua, script)?;

	// -- Check
	let expected = json!({
		"id":   [1, 2],
		"name": ["Alice", "Bob"]
		// 'email' omitted due to intersection
	});
	assert_eq!(res, expected);

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_records_to_columnar_row_not_table_err() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local recs = {
			  { id = 1, name = "Alice" },
			  "INVALID_ROW"
			}
			local ok, err = pcall(function()
			  return aip.shape.records_to_columnar(recs)
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
	assert_contains(&err_str, "aip.shape.records_to_columnar - Each record must be a table");

	Ok(())
}

#[tokio::test]
async fn test_lua_aip_shape_records_to_columnar_non_string_key_err() -> Result<()> {
	// -- Setup & Fixtures
	let lua = setup_lua(init_module, "shape").await?;
	let script = r#"
			local function make_bad()
			  local t = { id = 1, name = "Alice" }
			  t[123] = "bad" -- non-string key
			  return t
			end
			local recs = { make_bad() }
			local ok, err = pcall(function()
			  return aip.shape.records_to_columnar(recs)
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
	assert_contains(&err_str, "aip.shape.records_to_columnar - Record keys must be strings");

	Ok(())
}
