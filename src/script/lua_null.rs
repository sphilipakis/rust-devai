use mlua::{Error, MetaMethod, Result, UserData, UserDataMethods, Value};

#[derive(Clone, Copy, Debug)]
pub struct NullSentinel;

// implement display
impl std::fmt::Display for NullSentinel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "null")
	}
}

impl UserData for NullSentinel {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		// tostring -> "null"
		methods.add_meta_method(MetaMethod::ToString, |_, _, ()| Ok(String::from("null")));

		// Disallow reads/writes like null.x or null.x = y
		methods.add_meta_method(MetaMethod::Index, |_, _, (_k,): (Value,)| -> Result<Value> {
			Err(Error::RuntimeError("attempt to index null".into()))
		});
		methods.add_meta_method(MetaMethod::NewIndex, |_, _, (_k, _v): (Value, Value)| -> Result<()> {
			Err(Error::RuntimeError("attempt to modify null".into()))
		});

		// Optional: block iteration attempts
		methods.add_meta_method(MetaMethod::Pairs, |_, _, ()| -> Result<()> {
			Err(Error::RuntimeError("attempt to iterate over null".into()))
		});

		// NOTE: Somehow `MetaMethod::IPairs` not available for lua54: https://github.com/mlua-rs/mlua/issues/635
		// methods.add_meta_method(MetaMethod::IPairs, |_, _, ()| -> Result<()> {
		// 	Err(Error::RuntimeError("attempt to iterate over null".into()))
		// });
	}
}
