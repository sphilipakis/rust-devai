use mlua::{MetaMethod, UserData, UserDataMethods, Value};

#[derive(Clone, Copy, Debug)]
pub struct NASentinel;

impl UserData for NASentinel {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		// Pretty-print as "NA"
		methods.add_meta_method(MetaMethod::ToString, |_lua, _this, ()| Ok(String::from("NA")));

		// Optional: make it obviously non-table-like in Lua
		methods.add_meta_method(MetaMethod::Index, |_lua, _this, (_k,): (Value,)| {
			Err::<(), _>(mlua::Error::RuntimeError("attempt to index NA".into()))
		});
		methods.add_meta_method(MetaMethod::NewIndex, |_lua, _this, (_k, _v): (Value, Value)| {
			Err::<(), _>(mlua::Error::RuntimeError("attempt to modify NA".into()))
		});
		// You can add more metamethods here (Len, Call, etc.) to block other operations if desired.
	}
}
