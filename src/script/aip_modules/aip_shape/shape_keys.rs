use crate::Error;
use mlua::{Lua, Table, Value};

/// ## Lua Documentation
/// ---
/// Return a new record containing only the specified keys. The original record is not modified.
///
/// ```lua
/// -- API Signature
/// aip.shape.select_keys(rec: table, keys: string[]): table
/// ```
///
/// - Missing keys are ignored.
/// - If `keys` contains a non-string entry, an error is returned.
///
pub fn select_keys(lua: &Lua, rec: Table, keys: Table) -> mlua::Result<Value> {
	let out = lua.create_table()?;

	for (idx, key_val) in keys.sequence_values::<Value>().enumerate() {
		let key_val = key_val?;
		let key_str = match key_val {
			Value::String(s) => s,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.select_keys - Key names must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		};

		let val: Value = rec.get(key_str.clone())?;
		if !val.is_nil() {
			out.set(key_str, val)?;
		}
	}

	Ok(Value::Table(out))
}

///
/// Return a new record without the specified keys. The original record is not modified.
///
/// - Missing keys are ignored.
/// - If `keys` contains a non-string entry, an error is returned.
///
pub fn omit_keys(lua: &Lua, rec: Table, keys: Table) -> mlua::Result<Value> {
	use std::collections::HashSet;

	let mut omit_set: HashSet<String> = HashSet::new();
	for (idx, key_val) in keys.sequence_values::<Value>().enumerate() {
		let key_val = key_val?;
		match key_val {
			Value::String(s) => {
				omit_set.insert(s.to_string_lossy());
			}
			other => {
				return Err(Error::custom(format!(
					"aip.shape.omit_keys - Key names must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		}
	}

	let out = lua.create_table()?;
	for pair in rec.pairs::<Value, Value>() {
		let (k, v) = pair?;
		let skip = match &k {
			Value::String(s) => omit_set.contains(&s.to_string_lossy()),
			_ => false,
		};
		if !skip {
			out.set(k, v)?;
		}
	}

	Ok(Value::Table(out))
}

///
/// Return a new record containing only the specified keys and remove them from the original record (in-place).
///
/// - Missing keys are ignored.
/// - If `keys` contains a non-string entry, an error is returned.
///
pub fn extract_keys(lua: &Lua, rec: Table, keys: Table) -> mlua::Result<Value> {
	let out = lua.create_table()?;

	for (idx, key_val) in keys.sequence_values::<Value>().enumerate() {
		let key_val = key_val?;
		let key_str = match key_val {
			Value::String(s) => s,
			other => {
				return Err(Error::custom(format!(
					"aip.shape.extract_keys - Key names must be strings. Found '{}' at index {}",
					other.type_name(),
					idx + 1
				))
				.into());
			}
		};

		let val: Value = rec.get(key_str.clone())?;
		if !val.is_nil() {
			out.set(key_str.clone(), val)?;
			// Remove from original record (in-place)
			rec.set(key_str, Value::Nil)?;
		}
	}

	Ok(Value::Table(out))
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules::aip_shape::init_module;
	use serde_json::json;

	#[tokio::test]
	async fn test_lua_aip_shape_select_keys_simple() -> Result<()> {
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
	async fn test_lua_aip_shape_select_keys_missing_ignored() -> Result<()> {
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
	async fn test_lua_aip_shape_select_keys_rec_unchanged() -> Result<()> {
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
	async fn test_lua_aip_shape_select_keys_invalid_key_type_err() -> Result<()> {
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
	async fn test_lua_aip_shape_extract_keys_simple() -> Result<()> {
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
	async fn test_lua_aip_shape_extract_keys_missing_ignored() -> Result<()> {
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
	async fn test_lua_aip_shape_extract_keys_invalid_key_type_err() -> Result<()> {
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
}

// endregion: --- Tests
