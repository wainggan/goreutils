use crate::{lib_construct, library::Environment, types::Tagged};

pub trait EnvironmentStandalone: Environment {
	fn stdout(&self, write: &mut dyn FnMut(&mut dyn std::fmt::Write));
}

lib_construct!(lib_fn_print, EnvironmentStandalone, |stack, env| {
	while let Some(value) = stack() {
		env.stdout(&mut |f| _ = writeln!(f, "{:?}", value));
	}
	Tagged::None
});

#[cfg(test)]
mod test {
    use crate::{library::{Environment, standalone}, types::Tagged};

	struct Env<W: std::fmt::Write> {
		writer: std::sync::RwLock<W>,
	}

	impl<W: std::fmt::Write> Environment for Env<W> {}

	impl<W: std::fmt::Write> standalone::EnvironmentStandalone for Env<W> {
		fn stdout(&self, write: &mut dyn FnMut(&mut dyn std::fmt::Write)) {
			let mut lock = self.writer.write().expect("thread poisoned!");
			write(&mut *lock);
		}
	}

	fn stack(mut stack: Vec<Tagged>) -> impl FnMut() -> Option<Tagged> {
		move || {
			stack.pop()
		}
	}

	#[test]
	fn test_fn_print() {
		let env = Env {
			writer: std::sync::RwLock::new(String::new()),
		};

		standalone::lib_fn_print(&mut stack(vec![Tagged::Int(0)]), &env);

		let s = env.writer.read().expect("poisoned!");
		assert_eq!(s.as_str(), "Int(0)\n");
	}
}
