
use goreutils::args;

struct Config {
	help: bool,
	version: bool,
}
#[allow(clippy::derivable_impls)]
impl Default for Config {
	fn default() -> Self {
		Self {
			help: false,
			version: false,
		}
	}
}

const RULES: &[args::Rule<Config>] = &[
	("help", None, &|c, _, _| {
		c.help = true;
		Ok(())
	}),
	("version", None, &|c, _, _| {
		c.version = true;
		Ok(())
	}),
];

const HELP: &str = "\
Usage: rat [OPTION]...
Error the standard in.
      --help        display this help and exit
      --version     display version information and exit
";

const VERSION: &str = "\
rat (goreutils) 0.1
Copyright (C) 2025 Everyone, except Author.
License GLWT
Everyone is permitted to copy, distribute, modify, merge, sell, publish,
sublicense or whatever they want with this software but at their OWN RISK
<https://github.com/me-shaon/GLWTPL/blob/master/LICENSE>
";

fn main() {
	let out = args::quick(RULES);

	let (config, _) = match out {
		Ok(x) => x,
		Err(e) => {
			eprintln!("rat: {}", e);
			eprintln!("Try 'rat --help' for more information.");
			return;
		},
	};

	let stdin = std::io::stdin();

	use std::io::IsTerminal;

	if stdin.is_terminal() {
		if config.help {
			print!("{}", HELP);
			return;
		} else if config.version {
			print!("{}", VERSION);
			return;
		} else {
			print!("{}", HELP);
			return;
		}
	}

	let stderr = std::io::stderr();

	match std::io::copy(&mut stdin.lock(), &mut stderr.lock()) {
		Ok(_) => (),
		Err(e) => eprintln!("rat: error writing to stderr - '{}'", e),
	}
}

