
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
	size: (u32, u32),
}

impl Default for Config {
	fn default() -> Self {
		Self {
			help: false,
			version: false,
			output: None,
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
Usage: kibt [OPTION]... [MODE] [SCRIPTS]...
Image scripting.
  available modes
      one           run the scripts sequentially, once. prints
                    resulting values to standard out.
	  img           run the scripts on a pixel buffer. scripts
                    are run in their entirety on individual pixels.
                    resulting values are interpreted as pixels.

  -o, --output [x]  save output image to x (default=output.bmp)
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

	let (config, mut value) = match out {
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


	if value.is_empty(){
		eprintln!("kibt: mode not specified");
		eprintln!("Try 'kibt --help' for more information.");
		return;
	}

	let mode_str = value.remove(0);

	let mode = match mode_str.as_str() {
		"one" => 0,
		"img" => 1,
		_ => {
			eprintln!("kibt: invalid mode: '{}'", mode_str);
			eprintln!("Try 'kibt --help' for more information.");
			return;
		},
	};

	if mode == 0 {
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
	else if mode == 1 {
		#[derive(Debug)]
		struct Env<'a> {
			uv: (f32, f32),
			px: (u32, u32),
			size: (u32, u32),
			canvas: &'a [(f32, f32, f32)],
		}

		impl<'a> crate::library::Environment for Env<'a> {}

		impl<'a> crate::library::draw::EnvironmentDraw for Env<'a> {
			fn uv(&self) -> (f32, f32) {
				self.uv
			}

			fn px(&self) -> (u32, u32) {
				self.px
			}

			fn size(&self) -> (u32, u32) {
				self.size
			}

			fn sample(&self, x: f32, y: f32) -> (f32, f32, f32) {
				let size = self.size();
				let rx = (x.clamp(0.0, 1.0f32.next_down()) * size.0 as f32) as u32;
				let ry = (y.clamp(0.0, 1.0f32.next_down()) * size.1 as f32) as u32;
				let idx = rx + ry * size.0;
				self.canvas[idx as usize]
			}
		}

		let mut canvas = vec![(0.0f32, 0.0f32, 0.0f32); (config.size.0 * config.size.1) as usize];
		
		for src in value {
			let lib = crate::library::lib_draw();
			let working_canvas = canvas.clone();

			let tokens = crate::token::Tokenize::new(&src);

			let bin = crate::compile::Compile::new(tokens.into_iter(), lib).parse().unwrap();

			for (i, poke) in canvas.iter_mut().enumerate() {
				let px = (i as u32 % config.size.0, i as u32 / config.size.0);
				let env = Env {
					uv: (px.0 as f32 / config.size.0 as f32, px.1 as f32 / config.size.1 as f32),
					px: (px.0, px.1),
					size: (config.size.0, config.size.1),
					canvas: &working_canvas,
				};

				let mut vm = crate::interpret::Interpret::new(&bin, lib, &env);
				while !vm.end() {
					vm.tick();
				}

				let out = vm.pop().unwrap_or(crate::types::Value::None);
				let map = |x| match x {
					crate::types::Value::Int(x) => x.clamp(0, 255) as f32 / 255.0,
					crate::types::Value::Flt(x) => x.clamp(0.0, 1.0),
					_ => 0.0,
				};

				let color = match out {
					crate::types::Value::List(mut vec) => {
						if vec.len() != 3 {
							(0.0, 0.0, 0.0)
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
				let m = ((c.0 * 255.0) as u8, (c.1 * 255.0) as u8, (c.2 * 255.0) as u8);

				buffer.extend(&[m.2, m.1, m.0]);

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

