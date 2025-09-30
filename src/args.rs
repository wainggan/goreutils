
#[derive(Debug)]
pub enum Arg<'a> {
	Long(&'a str),
	Short(&'a str),
	Value(&'a str),
}
impl<'a> Arg<'a> {
	pub fn as_str(&self) -> &'a str {
		match self {
			Arg::Long(x) => x,
			Arg::Short(x) => x,
			Arg::Value(x) => x,
		}
	}
}

enum ParseState<'a> {
	Normal,
	Short(&'a str),
}

pub struct Parse<'a, I: Iterator<Item = &'a str>> {
	args: I,
	state: ParseState<'a>,
	rest: bool,
}
impl<'a, I: Iterator<Item = &'a str>> Parse<'a, I> {
	pub fn new(args: I) -> Self {
		Self {
			args,
			state: ParseState::Normal,
			rest: false,
		}
	}
}
impl<'a, I: Iterator<Item = &'a str>> Iterator for Parse<'a, I> {
	type Item = Arg<'a>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.state {
			ParseState::Normal => {
				let item = self.args.next()?;

				if self.rest {
					return Some(Arg::Value(item));
				}

				if item.starts_with("--") {
					if item == "--" {
						self.rest = true;
						self.next()
					} else {
						let item = item.trim_start_matches("--");
						Some(Arg::Long(item))
					}
				} else if item.starts_with("-") {
					let item = item.trim_start_matches("-");
					self.state = ParseState::Short(item);
					self.next()
				} else {
					Some(Arg::Value(item))
				}
			}
			ParseState::Short(ref mut item) => {
				let c = item.split_inclusive(|_| true).nth(0);

				if let Some(c) = c {
					// this is probably bad but I have absolutely no clue how to fix it
					*item = &item[1..];
					Some(Arg::Short(c))
				} else {
					self.state = ParseState::Normal;
					self.next()
				}
			}
		}
	}
}

pub type Rule<'a, T> = (
	// name
	&'static str,
	// short
	Option<char>,
	// mod
	&'a dyn Fn(
		// config
		&mut T,
		// params
		&mut dyn FnMut() -> Result<&'a str, ()>,
		// err
		&mut dyn std::fmt::Write
	) -> Result<(), ()>,
);

#[allow(clippy::result_unit_err)]
pub fn construct<'a, T: Default>(
	mut parse: impl Iterator<Item = Arg<'a>>,
	rules: &[Rule<'a, T>],
	err: &mut impl std::fmt::Write,
) -> Result<(T, Vec<&'a str>), ()> {
	let mut config = T::default();

	let mut values = Vec::new();

	while let Some(arg) = parse.next() {
		match arg {
			Arg::Long(x) => {
				let Some(rule) = rules.iter().find(|a| a.0 == x) else {
					write!(err, "unrecognized option '--{}'", x).map_err(|_| ())?;
					return Err(());
				};

				rule.2(
					&mut config,
					&mut || {
						if let Some(param) = parse.next() {
							Ok(param.as_str())
						} else {
							Err(())
						}
					},
					err,
				)?;
			}
			Arg::Short(x) => {
				let Some(rule) = rules.iter().find(|a| a.1 == x.chars().next()) else {
					write!(err, "invalid option -- '-{}'", x).map_err(|_| ())?;
					return Err(());
				};

				rule.2(
					&mut config,
					&mut || {
						if let Some(param) = parse.next() {
							Ok(param.as_str())
						} else {
							Err(())
						}
					},
					err,
				)?;
			}
			Arg::Value(x) => {
				values.push(x);
			}
		}
	}

	Ok((config, values))
}

pub fn quick<'a, T: Default>(rules: &[Rule<'a, T>]) -> Result<(T, Vec<String>), String> {
	let mut args = argv::iter().filter_map(|x| x.to_str());
	args.next();

	let mut err = String::new();	

	let config = construct(
		Parse::new(args),
		rules,
		&mut err,
	);

	config
		.map(|x| (x.0, x.1.iter().map(|x| x.to_string()).collect()))
		.map_err(|_| err)
}

