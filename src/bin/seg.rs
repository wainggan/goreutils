
use goreutils::args;

struct Config {
	help: bool,
	version: bool,
}
impl Default for Config {
	fn default() -> Self {
		Self {
			help: false,
			version: false,
		}
	}
}

const RULES: &[args::Rule<Config>] = &[
	("help", None, 0, &|c, _, _| {
		c.help = true;
		Ok(())
	}),
	("version", None, 0, &|c, _, _| {
		c.version = true;
		Ok(())
	}),
];

const HELP: &str = "\
Usage: seg [OPTION]...
Create a segmentation fault.
      --help        display this help and exit
      --version     display version information and exit
";

const VERSION: &str = "\
seg (goreutils) 0.1
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
			eprintln!("seg: {}", e);
			eprintln!("Try 'seg --help' for more information.");
			return;
		},
	};

	if config.help {
		print!("{}", HELP);
		return;
	}

	if config.version {
		print!("{}", VERSION);
		return;
	}

	let mut rng = lykoi_data::rng::XorShift64::new(69);

	loop {
		let a = rng.nextu();
		let b = a as usize;
		let c = b as *mut u8;
		unsafe {
			// pray to god this doesn't touch anything stupid
			std::ptr::write(c, 0);
		}
	}
}

