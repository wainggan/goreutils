/*
todo:
how the hell do you handle errors in rust
*/

use std::{fs, io::ErrorKind, os::unix::fs::FileExt, path::Path, usize};
use goreutils::args;

#[derive(Debug)]
enum ConfigMode {
	Poke,
	ShuffleInc(u8),
	ShuffleBit(u8),
	Swap,
}

#[derive(Debug)]
struct Config {
	help: bool,
	version: bool,
	verbose: bool,
	mode: ConfigMode,
	times: u32,
	range: Option<(usize, usize)>,
}
impl Default for Config {
	fn default() -> Self {
		Self {
			version: false,
			help: false,
			verbose: false,
			mode: ConfigMode::Poke,
			times: 1,
			range: None,
		}
	}
}

fn poke(rng: &mut lykoi_data::rng::XorShift64, path: &Path, config: &Config) {
	let file = fs::File::options().read(true).write(true).open(path);
	let file = match file {
		Ok(x) => x,
		Err(e) => {
			match e.kind() {
				ErrorKind::NotFound => eprintln!("px: cannot open '{:?}': No such file or directory", path.as_os_str()),
				ErrorKind::PermissionDenied => eprintln!("px: cannot open '{:?}': Permission denied", path.as_os_str()),
				_ => eprintln!("px: cannot open '{:?}': Unknown error", path.as_os_str()),
			}
			return;
		},
	};

	let file_meta = match file.metadata() {
		Ok(x) => x,
		Err(e) => {
			match e.kind() {
				ErrorKind::NotFound => eprintln!("px: cannot open '{:?}': No such file or directory", path.as_os_str()),
				ErrorKind::PermissionDenied => eprintln!("px: cannot open '{:?}': Permission denied", path.as_os_str()),
				_ => eprintln!("px: cannot open '{:?}': Unknown error", path.as_os_str()),
			}
			return;
		},
	};

	if config.verbose {
		println!(
			"{} '{:?}' {} time(s)",
			match config.mode {
				ConfigMode::Poke => "poking",
				ConfigMode::ShuffleBit(_) => "bit shuffling",
				ConfigMode::ShuffleInc(_) => "inc shuffling",
				ConfigMode::Swap => "swapping",
			},
			path.as_os_str(),
			config.times,
		);
	}

	let len = file_meta.len();
	let end = (len as usize).min(config.range.map(|x| x.1).unwrap_or(usize::MAX));
	let beg = 0.max(config.range.map(|x| x.0).unwrap_or(0)).min(end);

	let mut run = || {
		match config.mode {
			ConfigMode::Poke => {
				let offset = rng.range(beg as f64, end as f64) as u64;
				let data = (rng.nextf() * 256.0) as u8;

				match file.write_at(&[data], offset) {
					Ok(_) => (),
					Err(e) => {
						match e.kind() {
							_ => eprintln!("px: could not write to '{:?}'", path.as_os_str()),
						}
						return;
					},
				}
			},
			ConfigMode::Swap => {
				let offset_0 = rng.range(beg as f64, end as f64) as u64;
				let offset_1 = rng.range(beg as f64, end as f64) as u64;

				let mut scratch = [0];

				match file.read_at(&mut scratch, offset_0) {
					Ok(_) => (),
					Err(e) => {
						match e.kind() {
							_ => eprintln!("px: could not read '{:?}'", path.as_os_str()),
						}
						return;
					},
				};
				let data_0 = scratch[0];
				
				match file.read_at(&mut scratch, offset_1) {
					Ok(_) => (),
					Err(e) => {
						match e.kind() {
							_ => eprintln!("px: could not read '{:?}'", path.as_os_str()),
						}
						return;
					},
				};
				let data_1 = scratch[0];

				match file.write_at(&[data_0], offset_1) {
					Ok(_) => (),
					Err(e) => {
						match e.kind() {
							_ => eprintln!("px: could not write to '{:?}'", path.as_os_str()),
						}
						return;
					},
				}
				match file.write_at(&[data_1], offset_0) {
					Ok(_) => (),
					Err(e) => {
						match e.kind() {
							_ => eprintln!("px: could not write to '{:?}'", path.as_os_str()),
						}
						return;
					},
				}
			},
			ConfigMode::ShuffleBit(_) |
			ConfigMode::ShuffleInc(_) => {
				let offset = rng.range(beg as f64, end as f64) as u64;

				let mut scratch = [0];

				match file.read_at(&mut scratch, offset) {
					Ok(_) => (),
					Err(e) => {
						match e.kind() {
							_ => eprintln!("px: could not read '{:?}'", path.as_os_str()),
						}
						return;
					},
				};
				let mut data = scratch[0];

				match config.mode {
					ConfigMode::ShuffleInc(x) => {
						data = data.wrapping_add(x);
					}
					ConfigMode::ShuffleBit(0xff) => {
						let select_bit = rng.range(0.0, 8.0) as u8;
						let bit = 1u8 << select_bit;
						data ^= bit;
					}
					ConfigMode::ShuffleBit(x) => {
						let bit = 1u8 << x;
						data ^= bit;
					}
					_ => unreachable!(),
				}

				match file.write_at(&[data], offset) {
					Ok(_) => (),
					Err(e) => {
						match e.kind() {
							_ => eprintln!("px: could not write to '{:?}'", path.as_os_str()),
						}
						return;
					},
				}
			},
		}
	};
	
	for _ in 0..config.times {
		run();
	}
}

fn util_gen_time() -> u64 {
	std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap_or(std::time::Duration::from_millis(0x6969696969696969))
		.as_millis()
		.wrapping_pow(7)
		.wrapping_pow(5) as u64
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
	("verbose", Some('v'), &|c, _, _| {
		c.verbose = true;
		Ok(())
	}),
	("poke", Some('p'), &|c, _, _| {
		c.mode = ConfigMode::Poke;
		Ok(())
	}),
	("swap", Some('w'), &|c, _, _| {
		c.mode = ConfigMode::Swap;
		Ok(())
	}),
	("shuffle", Some('s'), &|c, a, e| {
		let Ok(x) = a() else {
			write!(e, "shuffle: missing parameter").map_err(|_| ())?;
			return Err(());
		};

		match x {
			"inc" => {
				let Ok(y) = a() else {
					write!(e, "shuffle: missing parameter").map_err(|_| ())?;
					return Err(());
				};
				let Ok(y) = u8::from_str_radix(y, 10) else {
					write!(e, "loop: unparsable input").map_err(|_| ())?;
					return Err(());
				};

				c.mode = ConfigMode::ShuffleInc(y);
			}
			"bit" => {
				let Ok(y) = a() else {
					write!(e, "shuffle: missing parameter").map_err(|_| ())?;
					return Err(());
				};

				if y == "_" {
					c.mode = ConfigMode::ShuffleBit(0xff);
				} else {
					let Ok(y) = u8::from_str_radix(y, 10) else {
						write!(e, "loop: unparsable input").map_err(|_| ())?;
						return Err(());
					};
					if y >= 8 {
						write!(e, "loop: why").map_err(|_| ())?;
						return Err(());
					}
					c.mode = ConfigMode::ShuffleBit(y);
				}
			}
			_ => {
				write!(e, "shuffle: unknown mode '{}'", x).map_err(|_| ())?;
				return Err(());
			}
		}

		Ok(())
	}),
	("loop", Some('l'), &|c, a, e| {
		let Ok(amount) = a() else {
			write!(e, "loop: missing parameter").map_err(|_| ())?;
			return Err(());
		};
		let Ok(amount) = u32::from_str_radix(amount, 10) else {
			write!(e, "loop: unparsable input").map_err(|_| ())?;
			return Err(());
		};
		c.times = amount;
		Ok(())
	}),
	("range", Some('r'), &|c, a, e| {
		let Ok(x) = a() else {
			write!(e, "range: missing minimum parameter").map_err(|_| ())?;
			return Err(());
		};
		let Ok(y) = a() else {
			write!(e, "range: missing maximum parameter").map_err(|_| ())?;
			return Err(());
		};

		let Ok(x) = usize::from_str_radix(x, 10) else {
			write!(e, "range: unparsable input").map_err(|_| ())?;
			return Err(());
		};
		let Ok(y) = usize::from_str_radix(y, 10) else {
			write!(e, "range: unparsable input").map_err(|_| ())?;
			return Err(());
		};
		
		c.range = Some((x, y));
		
		Ok(())
	}),
];

const HELP: &str = "\
Usage: px [OPTION]... [FILE]...
Edit a file fortuitously.
  -v, --verbose     list touched files
  -p, --poke        select a byte and randomize (default)
  -w, --swap        select two bytes and swap
  -s, --shuffle [x] [y]
                    select a byte and perform operation x
                    valid options for x:
                      inc - increments selected byte by y
                      bit - performs a bit-flip at bit y
                            (if y is '_', this is chosen
                            randomly)
  -r, --range [x] [y]
                    operate only between bytes x to y
  -l, --loop [x]    run operation x times (default=1)
      --help        display this help and exit
      --version     display version information and exit
";

const VERSION: &str = "\
px (goreutils) 0.1
Copyright (C) 2025 Everyone, except Author.
License GLWT
Everyone is permitted to copy, distribute, modify, merge, sell, publish,
sublicense or whatever they want with this software but at their OWN RISK
<https://github.com/me-shaon/GLWTPL/blob/master/LICENSE>
";

fn main() {

	let out = args::quick(RULES);

	let (config, mut paths) = match out {
		Ok(x) => x,
		Err(e) => {
			eprintln!("px: {}", e);
			eprintln!("Try 'px --help' for more information.");
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

	let mut rng = lykoi_data::rng::XorShift64::new(getrandom::u64().unwrap_or_else(|_| util_gen_time()));


	if paths.len() == 0 {
		paths.push(".".to_string());
	}

	for string in &paths {
		let path = Path::new(&string);
		let meta = match fs::metadata(path) {
			Ok(x) => x,
			Err(e) => {
				match e.kind() {
					ErrorKind::NotFound => eprintln!("px: cannot stat '{:?}': No such file or directory", path.as_os_str()),
					ErrorKind::PermissionDenied => eprintln!("px: cannot stat '{:?}': Permission denied", path.as_os_str()),
					_ => eprintln!("px: cannot stat '{:?}': Unknown error", path.as_os_str()),
				}
				return;
			},
		};
		
		if meta.is_dir() {

			let dirs = match fs::read_dir(path) {
				Ok(x) => x,
				Err(e) => {
					match e.kind() {
						ErrorKind::NotFound => eprintln!("px: cannot stat '{:?}': No such file or directory", path.as_os_str()),
						ErrorKind::PermissionDenied => eprintln!("px: cannot stat '{:?}': Permission denied", path.as_os_str()),
						_ => eprintln!("px: cannot stat '{:?}': Unknown error", path.as_os_str()),
					}
					return;
				},
			};

			for d in dirs {
				let d = match d {
					Ok(x) => x,
					Err(_) => {
						eprintln!("px: Unknown error");
						return;
					},
				};

				let meta = match d.metadata() {
					Ok(x) => x,
					Err(e) => {
						match e.kind() {
							ErrorKind::NotFound => eprintln!("px: cannot stat '{:?}': No such file or directory", path.as_os_str()),
							ErrorKind::PermissionDenied => eprintln!("px: cannot stat '{:?}': Permission denied", path.as_os_str()),
							_ => eprintln!("px: cannot stat '{:?}': Unknown error", path.as_os_str()),
						}
						return;
					},
				};

				if meta.is_file() {
					let path = d.path();
					poke(&mut rng, &path, &config);
				}
				// ignore nested directories
			}

		} else {
			poke(&mut rng, path, &config);
		}
	}
}

