use crate::{libconstruct, library::Environment, types::Value};

pub trait EnvironmentDraw: Environment {
	fn uv(&self) -> (f32, f32);
	fn px(&self) -> (u32, u32);
	fn size(&self) -> (u32, u32);
}

libconstruct!(lib_fn_uv_x, EnvironmentDraw, |_stack, env| {
	Value::Flt(env.uv().0)
});

libconstruct!(lib_fn_uv_y, EnvironmentDraw, |_stack, env| {
	Value::Flt(env.uv().1)
});

libconstruct!(lib_fn_px_x, EnvironmentDraw, |_stack, env| {
	Value::Int(env.px().0 as i32)
});

libconstruct!(lib_fn_px_y, EnvironmentDraw, |_stack, env| {
	Value::Int(env.px().1 as i32)
});

libconstruct!(lib_fn_width, EnvironmentDraw, |_stack, env| {
	Value::Int(env.size().0 as i32)
});

libconstruct!(lib_fn_height, EnvironmentDraw, |_stack, env| {
	Value::Int(env.size().1 as i32)
});

#[cfg(test)]
mod test {
    use crate::{library::{Environment, draw}, types::Value};

	struct Env {
		uv: (f32, f32),
		px: (u32, u32),
		size: (u32, u32),
	}

	impl Environment for Env {}

	impl draw::EnvironmentDraw for Env {
		fn uv(&self) -> (f32, f32) {
			self.uv
		}

		fn px(&self) -> (u32, u32) {
			self.px
		}

		fn size(&self) -> (u32, u32) {
			self.size
		}
	}

	const ENV: Env = Env {
		uv: (1.0, 2.0),
		px: (3, 4),
		size: (5, 6),
	};

	fn stack(mut stack: Vec<Value>) -> impl FnMut() -> Option<Value> {
		move || {
			stack.pop()
		}
	}

	#[test]
	fn test_fn_uv_x() {
		assert_eq!(draw::lib_fn_uv_x(&mut stack(vec![]), &ENV), Value::Flt(1.0));
	}

	#[test]
	fn test_fn_uv_y() {
		assert_eq!(draw::lib_fn_uv_y(&mut stack(vec![]), &ENV), Value::Flt(2.0));
	}

	#[test]
	fn test_fn_px_x() {
		assert_eq!(draw::lib_fn_px_x(&mut stack(vec![]), &ENV), Value::Int(3));
	}

	#[test]
	fn test_fn_px_y() {
		assert_eq!(draw::lib_fn_px_y(&mut stack(vec![]), &ENV), Value::Int(4));
	}

	#[test]
	fn test_fn_width() {
		assert_eq!(draw::lib_fn_width(&mut stack(vec![]), &ENV), Value::Int(5));
	}

	#[test]
	fn test_fn_height() {
		assert_eq!(draw::lib_fn_height(&mut stack(vec![]), &ENV), Value::Int(6));
	}
}


