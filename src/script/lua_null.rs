use mlua::{Error, MetaMethod, Result, UserData, UserDataMethods, Value};

const REPR: &str = "null";

#[derive(Clone, Copy, Debug)]
pub struct NullSentinel;

// implement display
impl std::fmt::Display for NullSentinel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{REPR}")
	}
}

impl UserData for NullSentinel {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		// tostring -> "null"
		methods.add_meta_method(MetaMethod::ToString, |_, _, ()| Ok(String::from(REPR)));

		// concat: works for both sides (null .. x) and (x .. null)
		methods.add_meta_function(MetaMethod::Concat, |_, (a, b): (Value, Value)| -> Result<String> {
			let to_piece = |v: Value| -> Result<String> {
				match v {
					Value::UserData(ud) if ud.is::<NullSentinel>() => Ok(REPR.to_string()),
					Value::String(s) => Ok(s.to_str()?.to_string()),
					Value::Integer(i) => Ok(i.to_string()),
					Value::Number(n) => Ok(n.to_string()),
					// Optional: allow these like tostring()
					Value::Boolean(b) => Ok(if b { "true" } else { "false" }.to_string()),
					Value::Nil => Ok("nil".to_string()),
					other => Err(Error::RuntimeError(format!(
						"attempt to concatenate a {} value",
						match other {
							Value::Table(_) => "table",
							Value::Function(_) => "function",
							Value::Thread(_) => "thread",
							Value::UserData(_) => "userdata",
							Value::LightUserData(_) => "light userdata",
							_ => "unknown",
						}
					))),
				}
			};

			Ok(to_piece(a)? + &to_piece(b)?)
		});

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
