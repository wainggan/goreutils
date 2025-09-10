/*
todo:
better argument parsing
how the hell do you handle errors in rust
*/

use std::{collections::BTreeMap, env, fs, io::ErrorKind, os::unix::fs::FileExt, path::Path};

use rand::{rngs::ThreadRng, Rng};

#[derive(Debug)]
enum ConfigMode {
	Random,
	Shuffle,
	Swap,
}

#[derive(Debug)]
struct Config {
	paths: Vec<String>,
	help: bool,
	version: bool,
	verbose: bool,
	mode: ConfigMode,
}
impl Default for Config {
	fn default() -> Self {
		Self {
			paths: vec![],
			version: false,
			help: false,
			verbose: false,
			mode: ConfigMode::Random,
		}
	}
}

#[derive(Debug)]
enum ConfigValid {
	Yes(Config),
	No(String),
}


fn poke(rng: &mut ThreadRng, path: &Path, config: &Config) {
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
		println!("poking '{:?}'", path.as_os_str());
	}

	let len = file_meta.len();

	match config.mode {
		ConfigMode::Random => {
			let offset = rng.random_range(0..len);
			let data = rng.random_range(0..255u8);

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
			let offset_0 = rng.random_range(0..len);
			let offset_1 = rng.random_range(0..len);

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
		ConfigMode::Shuffle => {
			let offset = rng.random_range(0..len);

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

			data = data.wrapping_add(1);

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
}


fn main() {
	let mut opts_big: BTreeMap<&'static str, &'static dyn Fn(&mut Config)> = BTreeMap::new();
	opts_big.insert("help", &|config: &mut Config| config.help = true);
	opts_big.insert("version", &|config: &mut Config| config.version = true);
	opts_big.insert("verbose", &|config: &mut Config| config.verbose = true);
	opts_big.insert("random", &|config: &mut Config| config.mode = ConfigMode::Random);
	opts_big.insert("swap", &|config: &mut Config| config.mode = ConfigMode::Swap);
	opts_big.insert("shuffle", &|config: &mut Config| config.mode = ConfigMode::Shuffle);

	let mut opts_small: BTreeMap<char, &'static dyn Fn(&mut Config)> = BTreeMap::new();
	opts_small.insert('v', &|config: &mut Config| config.verbose = true);
	opts_small.insert('r', &|config: &mut Config| config.mode = ConfigMode::Random);
	opts_small.insert('w', &|config: &mut Config| config.mode = ConfigMode::Swap);
	opts_small.insert('s', &|config: &mut Config| config.mode = ConfigMode::Shuffle);

	let mut config = ConfigValid::Yes(Config::default());
	
	let mut args = env::args_os();
	args.next();
	'error: for arg in args {
		let arg_raw = arg.to_str();
		let Some(arg_raw) = arg_raw else {
			continue;
		};

		if arg_raw.starts_with("--") {
			// big argument
			let arg_trim = arg_raw.trim_start_matches("--");

			let maybe = opts_big.get(arg_trim);
			
			if let Some(opt) = maybe {
				opt(match config {
					ConfigValid::Yes(ref mut x) => x,
					ConfigValid::No(_) => unreachable!(),
				});
			} else {
				config = ConfigValid::No(format!("unrecognized option '{}'", arg_raw));
				break 'error;
			}
		} else if arg_raw.starts_with("-") {
			// small argument
			let arg_trim = arg_raw.trim_start_matches("-");

			for c in arg_trim.chars() {
				let maybe = opts_small.get(&c);

				if let Some(opt) = maybe {
					opt(match config {
						ConfigValid::Yes(ref mut x) => x,
						ConfigValid::No(_) => unreachable!(),
					});
				} else {
					config = ConfigValid::No(format!("invalid option -- '{}'", c));
					break 'error;
				}
			}
		} else {
			match config {
				ConfigValid::Yes(ref mut x) => x.paths.push(arg_raw.to_string()),
				ConfigValid::No(_) => unreachable!(),
			}
		}
	}

	let mut config = match config {
		ConfigValid::Yes(x) => x,
		ConfigValid::No(x) => {
			eprintln!("px: {}", x);
			eprintln!("Try 'px --help' for more information.");
			return;
		},
	};

	if config.help {
		println!("Usage: px [OPTION]... [FILE]...");
		println!("Edit a file fortuitously.");
		println!("");
		println!("  -v, --verbose     list touched files");
		println!("  -r, --random      select a byte and randomize (default)");
		println!("  -w, --swap        select two bytes and swap");
		println!("  -s, --shuffle     select a byte and increment");
		println!("      --help        display this help and exit");
		println!("      --version     display version information and exit");
		return;
	}
	if config.version {
		println!("px (goreutils) 0.1");
		println!("Copyright (C) 2025 Everyone, except Author.");
		println!("License GLWT");
		println!("Everyone is permitted to copy, distribute, modify, merge, sell, publish,");
		println!("sublicense or whatever they want with this software but at their OWN RISK");
		println!("<https://github.com/me-shaon/GLWTPL/blob/master/LICENSE>");
		return;
	}

	let mut rng = rand::rng();


	if config.paths.len() == 0 {
		config.paths.push(".".to_string());
	}

	for string in &config.paths {
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

