use crate::{lib_construct, library::Environment, types::Tagged};

#[inline]
fn to_int(x: &Tagged) -> Tagged {
	match x {
		Tagged::Int(x) => Tagged::Int(*x),
		Tagged::Flt(x) => Tagged::Int(*x as i32),
		Tagged::Bool(x) => Tagged::Int(*x as i32),
		_ => Tagged::Int(0),
	}
}

#[inline]
fn to_flt(x: &Tagged) -> Tagged {
	match x {
		Tagged::Flt(x) => Tagged::Flt(*x),
		Tagged::Int(x) => Tagged::Flt(*x as f64),
		_ => Tagged::Flt(0.0),
	}
}

#[inline]
fn to_bool(x: &Tagged) -> Tagged {
	match x {
		Tagged::Bool(x) => Tagged::Bool(*x),
		Tagged::Flt(x) => Tagged::Bool(*x != 0.0),
		Tagged::Int(x) => Tagged::Bool(*x != 0),
		Tagged::None => Tagged::Bool(false),
		_ => Tagged::Bool(true),
	}
}

#[inline]
fn not(x: &Tagged) -> Tagged {
	match x {
		Tagged::Bool(x) => Tagged::Bool(!x),
		_ => Tagged::Bool(false),
	}
}

#[inline]
fn and(x: &Tagged, y: &Tagged) -> Tagged {
	match (x, y) {
		(Tagged::Bool(x), Tagged::Bool(y)) => Tagged::Bool(*x && *y),
		_ => Tagged::Bool(false),
	}
}

#[inline]
fn xor(x: &Tagged, y: &Tagged) -> Tagged {
	match (x, y) {
		(Tagged::Bool(x), Tagged::Bool(y)) => Tagged::Bool(*x ^ *y),
		_ => Tagged::Bool(false),
	}
}

#[inline]
fn or(x: &Tagged, y: &Tagged) -> Tagged {
	match (x, y) {
		(Tagged::Bool(x), Tagged::Bool(y)) => Tagged::Bool(*x || *y),
		_ => Tagged::Bool(false),
	}
}

#[inline]
fn eq(x: &Tagged, y: &Tagged) -> Tagged {
	Tagged::Bool(x == y)
}

#[inline]
fn ne(x: &Tagged, y: &Tagged) -> Tagged {
	Tagged::Bool(x != y)
}

#[inline]
fn lt(x: &Tagged, y: &Tagged) -> Tagged {
	match (x, y) {
		(Tagged::Int(x), Tagged::Int(y)) => Tagged::Bool(*x < *y),
		(Tagged::Flt(x), Tagged::Flt(y)) => Tagged::Bool(*x < *y),
		_ => Tagged::Bool(false),
	}
}

#[inline]
fn gt(x: &Tagged, y: &Tagged) -> Tagged {
	lt(y, x)
}

#[inline]
fn lte(x: &Tagged, y: &Tagged) -> Tagged {
	let a = lt(x, y);
	match a {
		Tagged::Bool(true) => eq(x, y),
		m => m,
	}
}

#[inline]
fn gte(x: &Tagged, y: &Tagged) -> Tagged {
	lte(y, x)
}

lib_construct!(lib_fn_int, Environment, |stack, _env| {
	to_int(&stack().unwrap_or(Tagged::None))
});

lib_construct!(lib_fn_flt, Environment, |stack, _env| {
	to_flt(&stack().unwrap_or(Tagged::None))
});

lib_construct!(lib_fn_bool, Environment, |stack, _env| {
	to_bool(&stack().unwrap_or(Tagged::None))
});

lib_construct!(lib_fn_not, Environment, |stack, _env| {
	not(&stack().unwrap_or(Tagged::None))
});

lib_construct!(lib_fn_neg, Environment, |stack, _env| {
	let Tagged = stack().unwrap_or(Tagged::None);
	match Tagged {
		Tagged::Int(x) => Tagged::Int(-x),
		Tagged::Flt(x) => Tagged::Flt(-x),
		x => x,
	}
});

macro_rules! impl_fn_cmp {
	($fn:ident, $stack:ident) => {
		{
			let mut x = $stack().unwrap_or(Tagged::None);
			let mut acc = Tagged::Bool(true);
			loop {
				let Some(y) = $stack() else {
					break;
				};
				acc = $fn(&x, &y);
				if let Tagged::Bool(false) = acc {
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
	let mut acc = Tagged::None;
	loop {
		let n = stack().unwrap_or(Tagged::None);
		if matches!(n, Tagged::None) {
			break;
		}
		acc = match (acc, n) {
			(Tagged::None, y) => y,
			(Tagged::Int(x), Tagged::Int(y)) => Tagged::Int(x.wrapping_add(y)),
			(Tagged::Flt(x), Tagged::Flt(y)) => Tagged::Flt(x + y),
			_ => Tagged::None,
		}
	}
	acc
});

lib_construct!(lib_fn_sub, Environment, |stack, _env| {
	let mut acc = Tagged::None;
	loop {
		let n = stack().unwrap_or(Tagged::None);
		if matches!(n, Tagged::None) {
			break;
		}
		acc = match (acc, n) {
			(Tagged::None, y) => y,
			(Tagged::Int(x), Tagged::Int(y)) => Tagged::Int(x.wrapping_sub(y)),
			(Tagged::Flt(x), Tagged::Flt(y)) => Tagged::Flt(x - y),
			_ => Tagged::None,
		}
	}
	acc
});

lib_construct!(lib_fn_mul, Environment, |stack, _env| {
	let mut acc = Tagged::None;
	loop {
		let n = stack().unwrap_or(Tagged::None);
		if matches!(n, Tagged::None) {
			break;
		}
		acc = match (acc, n) {
			(Tagged::None, y) => y,
			(Tagged::Int(x), Tagged::Int(y)) => Tagged::Int(x.wrapping_mul(y)),
			(Tagged::Flt(x), Tagged::Flt(y)) => Tagged::Flt(x * y),
			_ => Tagged::None,
		}
	}
	acc
});

lib_construct!(lib_fn_div, Environment, |stack, _env| {
	let mut acc = Tagged::None;
	loop {
		let n = stack().unwrap_or(Tagged::None);
		if matches!(n, Tagged::None) {
			break;
		}
		acc = match (acc, n) {
			(Tagged::None, y) => y,
			(Tagged::Int(x), Tagged::Int(y)) => Tagged::Int(x.wrapping_div(y)),
			(Tagged::Flt(x), Tagged::Flt(y)) => Tagged::Flt(x / y),
			_ => Tagged::None,
		}
	}
	acc
});

lib_construct!(lib_fn_rem, Environment, |stack, _env| {
	let Some(left) = stack() else {
		return Tagged::None;
	};
	let Some(right) = stack() else {
		return Tagged::None;
	};
	match (left, right) {
		(Tagged::Int(x), Tagged::Int(y)) => Tagged::Int(x % y),
		(Tagged::Flt(x), Tagged::Flt(y)) => Tagged::Flt(x % y),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_pow, Environment, |stack, _env| {
	let Some(left) = stack() else {
		return Tagged::None;
	};
	let Some(right) = stack() else {
		return Tagged::None;
	};
	match (left, right) {
		(Tagged::Int(x), Tagged::Int(y)) => Tagged::Int(x.pow(y.cast_unsigned())),
		(Tagged::Flt(x), Tagged::Flt(y)) => Tagged::Flt(x.powf(y)),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_cat, Environment, |stack, _env| {
	let mut acc = Tagged::None;
	loop {
		let n = stack().unwrap_or(Tagged::None);
		if matches!(n, Tagged::None) {
			break;
		}
		acc = match (acc, n) {
			(Tagged::None, y) => y,
			// (Tagged::Str(mut x), Tagged::Str(y)) => {
			// 	x.push_str(&y);
			// 	Tagged::Str(x)
			// },
			_ => Tagged::None,
		}
	}
	acc
});

lib_construct!(lib_fn_sin, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Tagged::None;
	};
	match x {
		Tagged::Flt(x) => Tagged::Flt(x.sin()),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_asin, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Tagged::None;
	};
	match x {
		Tagged::Flt(x) => Tagged::Flt(x.asin()),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_cos, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Tagged::None;
	};
	match x {
		Tagged::Flt(x) => Tagged::Flt(x.cos()),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_acos, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Tagged::None;
	};
	match x {
		Tagged::Flt(x) => Tagged::Flt(x.acos()),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_tan, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Tagged::None;
	};
	match x {
		Tagged::Flt(x) => Tagged::Flt(x.tan()),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_atan, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Tagged::None;
	};
	match x {
		Tagged::Flt(x) => Tagged::Flt(x.atan()),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_atan2, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Tagged::None;
	};
	let Some(y) = stack() else {
		return Tagged::None;
	};
	match (x, y) {
		(Tagged::Flt(x), Tagged::Flt(y)) => Tagged::Flt(x.atan2(y)),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_clamp, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Tagged::None;
	};
	let Some(y) = stack() else {
		return Tagged::None;
	};
	let Some(z) = stack() else {
		return Tagged::None;
	};
	match (x, y, z) {
		(Tagged::Flt(x), Tagged::None, Tagged::Flt(z)) => Tagged::Flt(x.max(z)),
		(Tagged::Flt(x), Tagged::Flt(y), Tagged::None) => Tagged::Flt(x.min(y)),
		(Tagged::Flt(x), Tagged::Flt(y), Tagged::Flt(z)) => Tagged::Flt(x.min(y).max(z)),
		(Tagged::Int(x), Tagged::None, Tagged::Int(z)) => Tagged::Int(x.max(z)),
		(Tagged::Int(x), Tagged::Int(y), Tagged::None) => Tagged::Int(x.min(y)),
		(Tagged::Int(x), Tagged::Int(y), Tagged::Int(z)) => Tagged::Int(x.min(y).max(z)),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_abs, Environment, |stack, _env| {
	let Some(x) = stack() else {
		return Tagged::None;
	};
	match x {
		Tagged::Int(x) => Tagged::Int(x.abs()),
		Tagged::Flt(x) => Tagged::Flt(x.abs()),
		_ => Tagged::None,
	}
});

lib_construct!(lib_fn_list, Environment, |stack, _env| {
	let mut vec = vec![];
	while let Some(Tagged) = stack() {
		vec.push(Tagged);
	}
	// Tagged::List(vec)
	todo!("unimplemented")
});

lib_construct!(lib_fn_pi, Environment, |_stack, _env| {
	Tagged::Flt(core::f64::consts::PI)
});

#[cfg(test)]
mod test {
    use crate::{library::{Environment, base}, types::Tagged};

	struct Env {}

	impl Environment for Env {}

	const ENV: Env = Env {};

	fn stack(mut stack: Vec<Tagged>) -> impl FnMut() -> Option<Tagged> {
		move || {
			stack.pop()
		}
	}

	#[test]
	fn test_fn_int() {
		assert_eq!(base::lib_fn_int(&mut stack(vec![Tagged::Int(2)]), &ENV), Tagged::Int(2));
		assert_eq!(base::lib_fn_int(&mut stack(vec![Tagged::Flt(2.5)]), &ENV), Tagged::Int(2));
		assert_eq!(base::lib_fn_int(&mut stack(vec![Tagged::None]), &ENV), Tagged::Int(0));
		assert_eq!(base::lib_fn_int(&mut stack(vec![]), &ENV), Tagged::Int(0));
	}

	#[test]
	fn test_fn_flt() {
		assert_eq!(base::lib_fn_flt(&mut stack(vec![Tagged::Flt(2.5)]), &ENV), Tagged::Flt(2.5));
		assert_eq!(base::lib_fn_flt(&mut stack(vec![Tagged::Int(2)]), &ENV), Tagged::Flt(2.0));
		assert_eq!(base::lib_fn_flt(&mut stack(vec![Tagged::None]), &ENV), Tagged::Flt(0.0));
		assert_eq!(base::lib_fn_flt(&mut stack(vec![]), &ENV), Tagged::Flt(0.0));
	}
}
