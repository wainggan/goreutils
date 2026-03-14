use crate::{lib_construct, library::Environment, types::Tagged};

pub trait EnvironmentDraw: Environment {
	fn uv(&self) -> (f64, f64);
	fn px(&self) -> (u32, u32);
	fn size(&self) -> (u32, u32);
	fn sample(&self, x: f64, y: f64) -> (f64, f64, f64);
}

lib_construct!(lib_fn_uv_x, EnvironmentDraw, |_stack, env| {
	Tagged::Flt(env.uv().0)
});

lib_construct!(lib_fn_uv_y, EnvironmentDraw, |_stack, env| {
	Tagged::Flt(env.uv().1)
});

lib_construct!(lib_fn_px_x, EnvironmentDraw, |_stack, env| {
	Tagged::Int(env.px().0 as i32)
});

lib_construct!(lib_fn_px_y, EnvironmentDraw, |_stack, env| {
	Tagged::Int(env.px().1 as i32)
});

lib_construct!(lib_fn_width, EnvironmentDraw, |_stack, env| {
	Tagged::Int(env.size().0 as i32)
});

lib_construct!(lib_fn_height, EnvironmentDraw, |_stack, env| {
	Tagged::Int(env.size().1 as i32)
});

lib_construct!(lib_fn_sample_r, EnvironmentDraw, |stack, env| {
	let pos_x = stack().unwrap_or_else(|| Tagged::Flt(env.uv().0));
	let pos_y = stack().unwrap_or_else(|| Tagged::Flt(env.uv().1));
	let col = match (pos_x, pos_y) {
		(Tagged::Flt(x), Tagged::Flt(y)) => env.sample(x, y),
		_ => return Tagged::None,
	};
	Tagged::Flt(col.0)
});

lib_construct!(lib_fn_sample_g, EnvironmentDraw, |stack, env| {
	let pos_x = stack().unwrap_or_else(|| Tagged::Flt(env.uv().0));
	let pos_y = stack().unwrap_or_else(|| Tagged::Flt(env.uv().1));
	let col = match (pos_x, pos_y) {
		(Tagged::Flt(x), Tagged::Flt(y)) => env.sample(x, y),
		_ => return Tagged::None,
	};
	Tagged::Flt(col.1)
});

lib_construct!(lib_fn_sample_b, EnvironmentDraw, |stack, env| {
	let pos_x = stack().unwrap_or_else(|| Tagged::Flt(env.uv().0));
	let pos_y = stack().unwrap_or_else(|| Tagged::Flt(env.uv().1));
	let col = match (pos_x, pos_y) {
		(Tagged::Flt(x), Tagged::Flt(y)) => env.sample(x, y),
		_ => return Tagged::None,
	};
	Tagged::Flt(col.2)
});

#[cfg(test)]
mod test {
    use crate::{library::{Environment, draw}, types::Tagged};

	struct Env {
		uv: (f64, f64),
		px: (u32, u32),
		size: (u32, u32),
	}

	impl Environment for Env {}

	impl draw::EnvironmentDraw for Env {
		fn uv(&self) -> (f64, f64) {
			self.uv
		}

		fn px(&self) -> (u32, u32) {
			self.px
		}

		fn size(&self) -> (u32, u32) {
			self.size
		}

		fn sample(&self, _x: f64, _y: f64) -> (f64, f64, f64) {
			(0.0, 0.0, 0.0)
		}
	}

	const ENV: Env = Env {
		uv: (1.0, 2.0),
		px: (3, 4),
		size: (5, 6),
	};

	fn stack(mut stack: Vec<Tagged>) -> impl FnMut() -> Option<Tagged> {
		move || {
			stack.pop()
		}
	}

	#[test]
	fn test_fn_uv_x() {
		assert_eq!(draw::lib_fn_uv_x(&mut stack(vec![]), &ENV), Tagged::Flt(1.0));
	}

	#[test]
	fn test_fn_uv_y() {
		assert_eq!(draw::lib_fn_uv_y(&mut stack(vec![]), &ENV), Tagged::Flt(2.0));
	}

	#[test]
	fn test_fn_px_x() {
		assert_eq!(draw::lib_fn_px_x(&mut stack(vec![]), &ENV), Tagged::Int(3));
	}

	#[test]
	fn test_fn_px_y() {
		assert_eq!(draw::lib_fn_px_y(&mut stack(vec![]), &ENV), Tagged::Int(4));
	}

	#[test]
	fn test_fn_width() {
		assert_eq!(draw::lib_fn_width(&mut stack(vec![]), &ENV), Tagged::Int(5));
	}

	#[test]
	fn test_fn_height() {
		assert_eq!(draw::lib_fn_height(&mut stack(vec![]), &ENV), Tagged::Int(6));
	}
}
