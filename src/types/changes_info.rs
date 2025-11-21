use mlua::{IntoLua, Lua, Value};

#[derive(Debug)]
pub struct ChangesInfo {
	/// Number of successful change count
	pub changed_count: i32,
	/// Change fail
	pub failed_changes: Vec<FailChange>,
}

impl IntoLua for ChangesInfo {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("changed_count", self.changed_count)?;
		if !self.failed_changes.is_empty() {
			let failed_changes_lua = lua.create_table()?;
			for (idx, item) in self.failed_changes.into_iter().enumerate() {
				failed_changes_lua.set(idx + 1, item)?;
			}
			table.set("failed_changes", failed_changes_lua)?;
		}
		Ok(Value::Table(table))
	}
}

#[derive(Debug)]
pub struct FailChange {
	pub search: String,
	pub replace: String,
	pub reason: String,
}

impl IntoLua for FailChange {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let table = lua.create_table()?;
		table.set("search", self.search)?;
		table.set("replace", self.replace)?;
		table.set("reason", self.reason)?;
		Ok(Value::Table(table))
	}
}
