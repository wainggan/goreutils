
use goreutils::args;

struct Config {
	help: bool,
	version: bool,
	delay: u64,
}
#[allow(clippy::derivable_impls)]
impl Default for Config {
	fn default() -> Self {
		Self {
			help: false,
			version: false,
			delay: 1000,
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
	("delay", Some('d'), &|c, a, e| {
		let Ok(x) = a() else {
			write!(e, "delay: missing parameter").map_err(|_| ())?;
			return Err(());
		};
		let Ok(x) = x.parse() else {
			write!(e, "delay: unparsable input").map_err(|_| ())?;
			return Err(());
		};

		c.delay = x;
		
		Ok(())
	}),
];

const HELP: &str = "\
Usage: stud [OPTION]... [FILE]
Efficiently print input slowly.
  -d, --delay [x]   delay in ms. (default=1000)
      --help        display this help and exit
      --version     display version information and exit
";

const VERSION: &str = "\
stud (goreutils) 0.1
Copyright (C) 2025 Everyone, except Author.
License GLWT
Everyone is permitted to copy, distribute, modify, merge, sell, publish,
sublicense or whatever they want with this software but at their OWN RISK
<https://github.com/me-shaon/GLWTPL/blob/master/LICENSE>
";

fn run<T: std::io::Read>(bytes: std::io::Bytes<T>, config: &Config) {
	let mut stdout = std::io::stdout();

	use std::io::Write;

	let time_total = config.delay;
	let time_sleep = config.delay / 3;

	for u in bytes {
		let Ok(u) = u else {
			break;
		};

		let now = std::time::Instant::now();

		if stdout.write(&[u]).is_err() {
			break;
		}
		if stdout.flush().is_err() {
			break;
		}

		if time_sleep != 0 {
			std::thread::sleep(std::time::Duration::from_millis(time_sleep));
		}

		while now.elapsed().as_millis() <= time_total.into() {}
	}
}

fn main() {
	let out = args::quick(RULES);

	let (config, paths) = match out {
		Ok(x) => x,
		Err(e) => {
			eprintln!("stud: {}", e);
			eprintln!("Try 'stud --help' for more information.");
			return;
		},
	};

	let stdin = std::io::stdin();

	use std::io::IsTerminal;
	use std::io::Read;

	if !stdin.is_terminal() {
		let bytes = std::io::BufReader::new(stdin).bytes();
		run(bytes, &config);
	} else {
		if config.help || paths.is_empty() {
			print!("{}", HELP);
			return;
		}
		if config.version {
			print!("{}", VERSION);
			return;
		}

		let path = std::path::Path::new(&paths[0]);

		let file = std::fs::File::options().read(true).write(true).open(path);
		let file = match file {
			Ok(x) => x,
			Err(e) => {
				match e.kind() {
					std::io::ErrorKind::NotFound => eprintln!("stud: cannot open '{:?}': No such file or directory", path.as_os_str()),
					std::io::ErrorKind::PermissionDenied => eprintln!("stud: cannot open '{:?}': Permission denied", path.as_os_str()),
					_ => eprintln!("stud: cannot open '{:?}': Unknown error", path.as_os_str()),
				}
				return;
			},
		};

		let bytes = std::io::BufReader::new(file).bytes();
		run(bytes, &config);
	}
}

