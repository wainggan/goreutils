
mod types;
mod token;
mod compile;
mod interpret;
mod library;


use std::io::Write;

use goreutils::args;

struct Config {
	help: bool,
	version: bool,
	output: Option<String>,
	one: bool,
	size: (u32, u32),

}

impl Default for Config {
	fn default() -> Self {
		Self {
			help: false,
			version: false,
			output: None,
			one: false,
			size: (64, 64),
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
	("output", Some('o'), &|c, a, e| {
		let Ok(x) = a() else {
			write!(e, "output: missing parameter").map_err(|_| ())?;
			return Err(());
		};
		c.output = Some(x.to_string());
		Ok(())
	}),
	("one", None, &|c, _, _| {
		c.one = true;
		Ok(())
	}),
	("size", Some('d'), &|c, a, e| {
		let Ok(x) = a() else {
			write!(e, "size: missing width parameter").map_err(|_| ())?;
			return Err(());
		};
		let Ok(y) = a() else {
			write!(e, "size: missing height parameter").map_err(|_| ())?;
			return Err(());
		};

		let Ok(x) = x.parse() else {
			write!(e, "size: unparsable width parameter").map_err(|_| ())?;
			return Err(());
		};
		let Ok(y) = y.parse() else {
			write!(e, "size: unparsable height parameter").map_err(|_| ())?;
			return Err(());
		};

		c.size = (x, y);
		
		Ok(())
	}),
];

const HELP: &str = "\
Usage: kibt [OPTION]...
Image scripting.
  -o, --output [x]  save output image to x (default=output.bmp)
      --one         outputs a single execution to standard out
  -d, --size [x] [y]
                    set canvas size to width=x and height=y
                    (default=64 64)
      --help        display this help and exit
      --version     display version information and exit
";

const VERSION: &str = "\
kibt (goreutils) 0.1
Copyright (C) 2025 Everyone, except Author.
License GLWT
Everyone is permitted to copy, distribute, modify, merge, sell, publish,
sublicense or whatever they want with this software but at their OWN RISK
<https://github.com/me-shaon/GLWTPL/blob/master/LICENSE>
";


fn main() {
	let out = args::quick(RULES);

	let (config, value) = match out {
		Ok(x) => x,
		Err(e) => {
			eprintln!("kibt: {}", e);
			eprintln!("Try 'kibt --help' for more information.");
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

	if config.one {
		#[derive(Debug)]
		struct Env {}

		impl crate::library::Environment for Env {}
		impl crate::library::standalone::EnvironmentStandalone for Env {
			fn stdout(&self, write: &mut dyn FnMut(&mut dyn std::fmt::Write)) {
				struct Adapt;

				impl std::fmt::Write for Adapt {
					fn write_str(&mut self, s: &str) -> std::fmt::Result {
						std::io::stdout().write_all(s.as_bytes()).or(Ok(()))
					}
				}

				write(&mut Adapt);
			}
		}

		for src in value {
			let env = Env {};

			let lib = crate::library::lib_standalone();

			let tokens = crate::token::Tokenize::new(&src);
	
			let bin = crate::compile::Compile::new(tokens, lib).parse().unwrap();

			let mut vm = crate::interpret::Interpret::new(&bin, lib, &env);
			while !vm.end() {
				vm.tick();
			}

			let out = vm.pop().unwrap_or(crate::types::Value::None);

			println!("{}", out);
		}
	}
	else {
		#[derive(Debug)]
		struct Env {
			uv: (f32, f32),
			px: (u32, u32),
			size: (u32, u32),
		}

		impl crate::library::Environment for Env {}

		impl crate::library::draw::EnvironmentDraw for Env {
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

		let mut canvas = vec![(0u8, 0u8, 0u8); (config.size.0 * config.size.1) as usize];

		let lib = crate::library::lib_draw();

		for src in value {
			let tokens = crate::token::Tokenize::new(&src);
			let bin = crate::compile::Compile::new(tokens.into_iter(), lib).parse().unwrap();

			for (i, poke) in canvas.iter_mut().enumerate() {
				let px = (i as u32 % config.size.0, i as u32 / config.size.0);
				let env = Env {
					uv: (px.0 as f32 / config.size.0 as f32, px.1 as f32 / config.size.1 as f32),
					px: (px.0, px.1),
					size: (config.size.0, config.size.1),
				};

				let mut vm = crate::interpret::Interpret::new(&bin, lib, &env);
				while !vm.end() {
					vm.tick();
				}

				let out = vm.pop().unwrap_or(crate::types::Value::None);
				
				let map = |x| match x {
					crate::types::Value::Int(x) => x.clamp(0, 255) as u8,
					crate::types::Value::Flt(x) => (x.clamp(0.0, 1.0) * 256.0) as u8,
					_ => 0,
				};

				let color = match out {
					crate::types::Value::List(mut vec) => {
						if vec.len() != 3 {
							(0, 0, 0)
						} else {
							let b = vec.pop().unwrap();
							let g = vec.pop().unwrap();
							let r = vec.pop().unwrap();
							(map(r), map(g), map(b))
						}
					}
					_ => {
						let x = map(out);
						(x, x, x)
					}
				};

				*poke = color;
			}
		}

		let mut buffer = vec![];

		// https://www.ece.ualberta.ca/~elliott/ee552/studentAppNotes/2003_w/misc/bmp_file_format/bmp_file_format.htm

		// magic
		buffer.extend(b"BM");
		// size (todo)
		buffer.extend(&[0, 0, 0, 0]);
		// reserved
		buffer.extend(&0u32.to_le_bytes());
		// offset
		buffer.extend(&(14u32 + 40).to_le_bytes());

		// info header size
		buffer.extend(&40u32.to_le_bytes());
		// bitmap width
		buffer.extend(&config.size.0.to_le_bytes());
		// bitmap height
		buffer.extend(&config.size.1.to_le_bytes());
		// planes
		buffer.extend(&1u16.to_le_bytes());
		// bits per pixel
		buffer.extend(&24u16.to_le_bytes());
		// compression
		buffer.extend(&0u32.to_le_bytes());
		// image size
		buffer.extend(&0u32.to_le_bytes());
		// xp/m
		buffer.extend(&0u32.to_le_bytes());
		// yp/m
		buffer.extend(&0u32.to_le_bytes());
		// colors used
		buffer.extend(&(0xffu32 * 0xff * 0xff).to_le_bytes());
		// important colors
		buffer.extend(&0u32.to_le_bytes());

		let mut y = config.size.1;
		while y > 0 {
			y -= 1;

			let mut x = 0;
			while x < config.size.0 {
				let i = x + y * config.size.0;

				let c = &canvas[i as usize];

				buffer.extend(&[c.2, c.1, c.0]);

				x += 1;

				if x >= config.size.0 {
					let fold = config.size.0 & 0b11;
					buffer.extend(std::iter::repeat_n(0, fold as usize));
				}
			}
		}

		std::fs::write(
			config.output.as_deref()
				.unwrap_or("output.bmp"),
			&buffer
		).unwrap();
	}
}

