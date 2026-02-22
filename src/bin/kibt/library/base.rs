use crate::{libconstruct, library::Environment, types::Value};

libconstruct!(lib_fn_int, Environment, |stack, _env| {
	match stack().unwrap_or(Value::None) {
		Value::Int(x) => Value::Int(x),
		Value::Flt(x) => Value::Int(x as i32),
		_ => Value::Int(0),
	}
});

libconstruct!(lib_fn_flt, Environment, |stack, _env| {
	match stack().unwrap_or(Value::None) {
		Value::Flt(x) => Value::Flt(x),
		Value::Int(x) => Value::Flt(x as f32),
		_ => Value::Flt(0.0),
	}
});

libconstruct!(lib_fn_not, Environment, |stack, _env| {
	let value = stack().unwrap_or(Value::None);
	Value::Int(match value {
		Value::Int(x) => (x == 0) as i32,
		_ => 0
	})
});

libconstruct!(lib_fn_neg, Environment, |stack, _env| {
	let value = stack().unwrap_or(Value::None);
	match value {
		Value::Int(x) => Value::Int(-x),
		Value::Flt(x) => Value::Flt(-x),
		x => x,
	}
});

libconstruct!(lib_fn_cmp, Environment, |stack, _env| {
	let mut acc = stack().unwrap_or(Value::None);
	let mut check = true;
	loop {
		let n = stack().unwrap_or(Value::None);
		if matches!(n, Value::None) {
			break;
		}
		check = check && match (&acc, &n) {
			(Value::Int(x), Value::Int(y)) => x < y,
			(Value::Flt(x), Value::Flt(y)) => x < y,
			_ => false,
		};
		if !check {
			break;
		}
		acc = n;
	}
	Value::Int(if check { 1 } else { 0 })
});

libconstruct!(lib_fn_eq, Environment, |stack, _env| {
	let acc = stack().unwrap_or(Value::None);
	let mut check = true;
	loop {
		let n = stack().unwrap_or(Value::None);
		if matches!(n, Value::None) {
			break;
		}
		check = check && acc == n;
	}
	Value::Int(if check { 1 } else { 0 })
});

libconstruct!(lib_fn_add, Environment, |stack, _env| {
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

libconstruct!(lib_fn_sub, Environment, |stack, _env| {
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

libconstruct!(lib_fn_mul, Environment, |stack, _env| {
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

libconstruct!(lib_fn_div, Environment, |stack, _env| {
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

libconstruct!(lib_fn_list, Environment, |stack, _env| {
	let mut vec = vec![];
	while let Some(value) = stack() {
		vec.push(value);
	}
	Value::List(vec)
});

libconstruct!(lib_fn_pi, Environment, |_stack, _env| {
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

