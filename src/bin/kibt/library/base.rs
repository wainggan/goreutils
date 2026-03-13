use crate::{lib_construct, library::Environment, types::Value};

#[inline]
fn to_int(x: &Value) -> Value {
	match x {
		Value::Int(x) => Value::Int(*x),
		Value::Flt(x) => Value::Int(*x as i32),
		Value::Bool(x) => Value::Int(*x as i32),
		_ => Value::Int(0),
	}
}

#[inline]
fn to_flt(x: &Value) -> Value {
	match x {
		Value::Flt(x) => Value::Flt(*x),
		Value::Int(x) => Value::Flt(*x as f32),
		_ => Value::Flt(0.0),
	}
}

#[inline]
fn to_bool(x: &Value) -> Value {
	match x {
		Value::Bool(x) => Value::Bool(*x),
		Value::Flt(x) => Value::Bool(*x != 0.0),
		Value::Int(x) => Value::Bool(*x != 0),
		Value::None => Value::Bool(false),
		_ => Value::Bool(true),
	}
}

#[inline]
fn not(x: &Value) -> Value {
	match x {
		Value::Bool(x) => Value::Bool(!x),
		_ => Value::Bool(false),
	}
}

#[inline]
fn and(x: &Value, y: &Value) -> Value {
	match (x, y) {
		(Value::Bool(x), Value::Bool(y)) => Value::Bool(*x && *y),
		_ => Value::Bool(false),
	}
}

#[inline]
fn xor(x: &Value, y: &Value) -> Value {
	match (x, y) {
		(Value::Bool(x), Value::Bool(y)) => Value::Bool(*x ^ *y),
		_ => Value::Bool(false),
	}
}

#[inline]
fn or(x: &Value, y: &Value) -> Value {
	match (x, y) {
		(Value::Bool(x), Value::Bool(y)) => Value::Bool(*x || *y),
		_ => Value::Bool(false),
	}
}

#[inline]
fn eq(x: &Value, y: &Value) -> Value {
	Value::Bool(x == y)
}

#[inline]
fn ne(x: &Value, y: &Value) -> Value {
	Value::Bool(x != y)
}

#[inline]
fn lt(x: &Value, y: &Value) -> Value {
	match (x, y) {
		(Value::Int(x), Value::Int(y)) => Value::Bool(*x < *y),
		(Value::Flt(x), Value::Flt(y)) => Value::Bool(*x < *y),
		_ => Value::Bool(false),
	}
}

#[inline]
fn gt(x: &Value, y: &Value) -> Value {
	lt(y, x)
}

#[inline]
fn lte(x: &Value, y: &Value) -> Value {
	let a = lt(x, y);
	match a {
		Value::Bool(true) => eq(x, y),
		m => m,
	}
}

#[inline]
fn gte(x: &Value, y: &Value) -> Value {
	lte(y, x)
}

lib_construct!(lib_fn_int, Environment, |stack, _env| {
	to_int(&stack().unwrap_or(Value::None))
});

lib_construct!(lib_fn_flt, Environment, |stack, _env| {
	to_flt(&stack().unwrap_or(Value::None))
});

lib_construct!(lib_fn_bool, Environment, |stack, _env| {
	to_bool(&stack().unwrap_or(Value::None))
});

lib_construct!(lib_fn_not, Environment, |stack, _env| {
	not(&stack().unwrap_or(Value::None))
});

lib_construct!(lib_fn_neg, Environment, |stack, _env| {
	let value = stack().unwrap_or(Value::None);
	match value {
		Value::Int(x) => Value::Int(-x),
		Value::Flt(x) => Value::Flt(-x),
		x => x,
	}
});

macro_rules! impl_fn_cmp {
	($fn:ident, $stack:ident) => {
		{
			let mut x = $stack().unwrap_or(Value::None);
			let mut acc = Value::Bool(true);
			loop {
				let Some(y) = $stack() else {
					break;
				};
				acc = $fn(&x, &y);
				if let Value::Bool(false) = acc {
					break;
				}
				x = y;
			}
			acc
		}
	};
}

lib_construct!(lib_fn_or, Environment, |stack, _env| {
	impl_fn_cmp!(or, stack)
});

lib_construct!(lib_fn_and, Environment, |stack, _env| {
	impl_fn_cmp!(and, stack)
});

lib_construct!(lib_fn_xor, Environment, |stack, _env| {
	impl_fn_cmp!(xor, stack)
});

lib_construct!(lib_fn_lt, Environment, |stack, _env| {
	impl_fn_cmp!(lt, stack)
});

lib_construct!(lib_fn_gt, Environment, |stack, _env| {
	impl_fn_cmp!(gt, stack)
});

lib_construct!(lib_fn_lte, Environment, |stack, _env| {
	impl_fn_cmp!(lte, stack)
});

lib_construct!(lib_fn_gte, Environment, |stack, _env| {
	impl_fn_cmp!(gte, stack)
});

lib_construct!(lib_fn_eq, Environment, |stack, _env| {
	impl_fn_cmp!(eq, stack)
});

lib_construct!(lib_fn_ne, Environment, |stack, _env| {
	impl_fn_cmp!(ne, stack)
});

lib_construct!(lib_fn_add, Environment, |stack, _env| {
	let mut acc = Value::None;
	loop {
		let n = stack().unwrap_or(Value::None);
		if matches!(n, Value::None) {
			break;
		}
		acc = match (acc, n) {
			(Value::None, y) => y,
			(Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_add(y)),
			(Value::Flt(x), Value::Flt(y)) => Value::Flt(x + y),
			_ => Value::None,
		}
	}
	acc
});

lib_construct!(lib_fn_sub, Environment, |stack, _env| {
	let mut acc = Value::None;
	loop {
		let n = stack().unwrap_or(Value::None);
		if matches!(n, Value::None) {
			break;
		}
		acc = match (acc, n) {
			(Value::None, y) => y,
			(Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_sub(y)),
			(Value::Flt(x), Value::Flt(y)) => Value::Flt(x - y),
			_ => Value::None,
		}
	}
	acc
});

lib_construct!(lib_fn_mul, Environment, |stack, _env| {
	let mut acc = Value::None;
	loop {
		let n = stack().unwrap_or(Value::None);
		if matches!(n, Value::None) {
			break;
		}
		acc = match (acc, n) {
			(Value::None, y) => y,
			(Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_mul(y)),
			(Value::Flt(x), Value::Flt(y)) => Value::Flt(x * y),
			_ => Value::None,
		}
	}
	acc
});

lib_construct!(lib_fn_div, Environment, |stack, _env| {
	let mut acc = Value::None;
	loop {
		let n = stack().unwrap_or(Value::None);
		if matches!(n, Value::None) {
			break;
		}
		acc = match (acc, n) {
			(Value::None, y) => y,
			(Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_div(y)),
			(Value::Flt(x), Value::Flt(y)) => Value::Flt(x / y),
			_ => Value::None,
		}
	}
	acc
});

lib_construct!(lib_fn_rem, Environment, |stack, _env| {
	let Some(left) = stack() else {
		return Value::None;
	};
	let Some(right) = stack() else {
		return Value::None;
	};
	match (left, right) {
		(Value::Int(x), Value::Int(y)) => Value::Int(x % y),
		(Value::Flt(x), Value::Flt(y)) => Value::Flt(x % y),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_pow, Environment, |stack, _env| {
	let Some(left) = stack() else {
		return Value::None;
	};
	let Some(right) = stack() else {
		return Value::None;
	};
	match (left, right) {
		(Value::Int(x), Value::Int(y)) => Value::Int(x.pow(y.cast_unsigned())),
		(Value::Flt(x), Value::Flt(y)) => Value::Flt(x.powf(y)),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_cat, Environment, |stack, _env| {
	let mut acc = Value::None;
	loop {
		let n = stack().unwrap_or(Value::None);
		if matches!(n, Value::None) {
			break;
		}
		acc = match (acc, n) {
			(Value::None, y) => y,
			(Value::Str(mut x), Value::Str(y)) => {
				x.push_str(&y);
				Value::Str(x)
			},
			_ => Value::None,
		}
	}
	acc
});

lib_construct!(lib_fn_sin, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Value::None;
	};
	match x {
		Value::Flt(x) => Value::Flt(x.sin()),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_asin, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Value::None;
	};
	match x {
		Value::Flt(x) => Value::Flt(x.asin()),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_cos, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Value::None;
	};
	match x {
		Value::Flt(x) => Value::Flt(x.cos()),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_acos, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Value::None;
	};
	match x {
		Value::Flt(x) => Value::Flt(x.acos()),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_tan, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Value::None;
	};
	match x {
		Value::Flt(x) => Value::Flt(x.tan()),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_atan, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Value::None;
	};
	match x {
		Value::Flt(x) => Value::Flt(x.atan()),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_atan2, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Value::None;
	};
	let Some(y) = stack() else {
		return Value::None;
	};
	match (x, y) {
		(Value::Flt(x), Value::Flt(y)) => Value::Flt(x.atan2(y)),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_clamp, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Value::None;
	};
	let Some(y) = stack() else {
		return Value::None;
	};
	let Some(z) = stack() else {
		return Value::None;
	};
	match (x, y, z) {
		(Value::Flt(x), Value::None, Value::Flt(z)) => Value::Flt(x.max(z)),
		(Value::Flt(x), Value::Flt(y), Value::None) => Value::Flt(x.min(y)),
		(Value::Flt(x), Value::Flt(y), Value::Flt(z)) => Value::Flt(x.min(y).max(z)),
		(Value::Int(x), Value::None, Value::Int(z)) => Value::Int(x.max(z)),
		(Value::Int(x), Value::Int(y), Value::None) => Value::Int(x.min(y)),
		(Value::Int(x), Value::Int(y), Value::Int(z)) => Value::Int(x.min(y).max(z)),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_abs, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Value::None;
	};
	match x {
		Value::Int(x) => Value::Int(x.abs()),
		Value::Flt(x) => Value::Flt(x.abs()),
		_ => Value::None,
	}
});

lib_construct!(lib_fn_list, Environment, |stack, _env| {
	let mut vec = vec![];
	while let Some(value) = stack() {
		vec.push(value);
	}
	Value::List(vec)
});

lib_construct!(lib_fn_pi, Environment, |_stack, _env| {
	Value::Flt(core::f32::consts::PI)
});

#[cfg(test)]
mod test {
    use crate::{library::{Environment, base}, types::Value};

	struct Env {}

	impl Environment for Env {}

	const ENV: Env = Env {};

	fn stack(mut stack: Vec<Value>) -> impl FnMut() -> Option<Value> {
		move || {
			stack.pop()
		}
	}

	#[test]
	fn test_fn_int() {
		assert_eq!(base::lib_fn_int(&mut stack(vec![Value::Int(2)]), &ENV), Value::Int(2));
		assert_eq!(base::lib_fn_int(&mut stack(vec![Value::Flt(2.5)]), &ENV), Value::Int(2));
		assert_eq!(base::lib_fn_int(&mut stack(vec![Value::None]), &ENV), Value::Int(0));
		assert_eq!(base::lib_fn_int(&mut stack(vec![]), &ENV), Value::Int(0));
	}

	#[test]
	fn test_fn_flt() {
		assert_eq!(base::lib_fn_flt(&mut stack(vec![Value::Flt(2.5)]), &ENV), Value::Flt(2.5));
		assert_eq!(base::lib_fn_flt(&mut stack(vec![Value::Int(2)]), &ENV), Value::Flt(2.0));
		assert_eq!(base::lib_fn_flt(&mut stack(vec![Value::None]), &ENV), Value::Flt(0.0));
		assert_eq!(base::lib_fn_flt(&mut stack(vec![]), &ENV), Value::Flt(0.0));
	}
}

