// todo: change String to &str

pub type Parameter<T> = (
	&'static str,
	Option<char>,
	u8,
	&'static dyn Fn(&mut T, &[String], &mut dyn std::fmt::Write) -> Result<(), ()>,
);

#[derive(Debug)]
pub struct ParseResult<T> {
	pub exe: String,
	pub mode: T,
	pub path: Vec<String>,
}

pub fn parse<T: Default>(
	def: &[Parameter<T>],
	mut args: impl Iterator<Item = String>,
	err: &mut impl std::fmt::Write
) -> Result<ParseResult<T>, ()> {
	let Some(exe) = args.next() else {
		write!(err, "unknown error").map_err(|_| ())?;
		return Err(());
	};

	let mut default = T::default();

	let mut path = vec![];
	let mut parse_files = false;

	while let Some(arg) = args.next() {
		let s = arg.as_str();

		if parse_files {
			path.push(arg);
		} else if s.starts_with("--") {
			if s == "--" {
				parse_files = true;
			} else {
				let s = s.trim_start_matches("--");

				let mut valid = false;

				for opt in def {
					if opt.0 == s {
						let mut params = vec![];

						for _ in 0..opt.2 {
							let Some(param) = args.next() else {
								write!(err, "missing operand for '{}'", opt.0).map_err(|_| ())?;
								return Err(());
							};
							params.push(param);
						}

						opt.3(&mut default, &params, err)?;
						valid = true;
						break;
					}
				}

				if !valid {
					write!(err, "unknown parameter '{}'", s).map_err(|_| ())?;
					return Err(());
				}
			}
		} else if s.starts_with("-") {
			let s = s.trim_start_matches("-");

			let total = s.chars().count();

			let mut chars = s.char_indices();

			while let Some((i, c)) = chars.next() {
				let mut valid = false;

				for opt in def {
					if let Some(check) = opt.1 && check == c {
						let mut params = vec![];

						if opt.2 > 0 {
							if i == total - 1 {
								for _ in 0..opt.2 {
									let Some(param) = args.next() else {
										write!(err, "missing operand for '{}' ('{}')", c, opt.0).map_err(|_| ())?;
										return Err(());
									};
									params.push(param);
								}
							} else {
								write!(err, "missing operand for '{}' ('{}')", c, opt.0).map_err(|_| ())?;
								return Err(());
							}
						}

						opt.3(&mut default, &params, err)?;
						valid = true;
						break;
					}
				}

				if !valid {
					write!(err, "unknown parameter '{}'", s).map_err(|_| ())?;
					return Err(());
				}
			}
		} else {
			parse_files = true;
			path.push(arg);
		}
	}

	Ok(ParseResult {
		exe,
		mode: default,
		path,
	})
}
