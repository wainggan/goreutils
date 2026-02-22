







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
		for src in value {
			let env = Environment {
				uv: (0.0, 0.0),
				px: (0, 0),
				size: (1, 1),
			};

			let tokens = crate::Tokenize::new(&src);
	
			let bin = crate::Compile::new(tokens, GLOBALS).parse().unwrap();

			let mut vm = crate::Interpret::new(&bin, GLOBALS, &env);
			while !vm.end() {
				vm.tick();
			}

			let out = vm.stack.pop().unwrap_or(Value::None);

			println!("{}", out);
		}
	}
	else {
		let mut canvas = vec![(0u8, 0u8, 0u8); (config.size.0 * config.size.1) as usize];

		for src in value {
			let tokens = crate::Tokenize::new(&src);
			let bin = crate::Compile::new(tokens.into_iter(), GLOBALS).parse().unwrap();

			for (i, poke) in canvas.iter_mut().enumerate() {
				let px = (i as u32 % config.size.0, i as u32 / config.size.0);
				let env = Environment {
					uv: (px.0 as f32 / config.size.0 as f32, px.1 as f32 / config.size.1 as f32),
					px: (px.0, px.1),
					size: (config.size.0, config.size.1),
				};

				let mut vm = crate::Interpret::new(&bin, GLOBALS, &env);
				while !vm.end() {
					vm.tick();
				}

				let out = vm.stack.pop().unwrap_or(Value::None);
				
				let map = |x| match x {
					Value::Int(x) => x.clamp(0, 255) as u8,
					Value::Flt(x) => (x.clamp(0.0, 1.0) * 256.0) as u8,
					_ => 0,
				};

				let color = match out {
					Value::List(mut vec) => {
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


#[cfg(test)]
mod test {
    use crate::{Token, Kind};

	#[test]
	fn test_tokens() {
		let src = "(0 0.0 test) (0)";
		let vec = crate::Tokenize::new(src).collect::<Vec<_>>();
		assert_eq!(
			&vec,
			&[
				Token { kind: Kind::LParen, data: "(" },
				Token { kind: Kind::Int, data: "0" },
				Token { kind: Kind::Flt, data: "0.0" },
				Token { kind: Kind::Ident, data: "test" },
				Token { kind: Kind::RParen, data: ")" },
				Token { kind: Kind::LParen, data: "(" },
				Token { kind: Kind::Int, data: "0" },
				Token { kind: Kind::RParen, data: ")" },
			],
		);
	}

	#[test]
	fn test_interpret_0() {
		let src = "{1 0.2}";
		
		let tokens = crate::Tokenize::new(src).collect::<Vec<_>>();
		println!("{:?}", tokens);

		let globals = [];

		let bin = crate::Compile::new(tokens.into_iter(), &globals).parse().unwrap();

		assert_eq!(&bin, &[
			crate::ops::LIT_INT,
			0, 0, 0, 1,
			crate::ops::POP,
			crate::ops::LIT_FLT,
			62, 76, 204, 205,
		]);

		let mut interpret = crate::Interpret::new(&bin, &globals, &());

		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(1)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Flt(0.2)]);
		interpret.tick();
	}

	#[test]
	fn test_interpret_1() {
		let src = "{ let a 1 let b 2 a b }";
		
		let tokens = crate::Tokenize::new(src).collect::<Vec<_>>();
		println!("{:?}", tokens);

		let globals = [];

		let bin = crate::Compile::new(tokens.into_iter(), &globals).parse().unwrap();

		assert_eq!(&bin, &[
			crate::ops::LIT_INT,
			0, 0, 0, 1,
			crate::ops::LIT_NONE,
			crate::ops::POP,

			crate::ops::LIT_INT,
			0, 0, 0, 2,
			crate::ops::LIT_NONE,
			crate::ops::POP,
			
			crate::ops::GET,
			0,
			crate::ops::POP,

			crate::ops::GET,
			1,
			crate::ops::SWAP,
			crate::ops::POP,

			crate::ops::SWAP,
			crate::ops::POP,
		]);

		let mut interpret = crate::Interpret::new(&bin, &globals, &());

		interpret.tick();
		interpret.tick();
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(1)]);
		interpret.tick();
		interpret.tick();
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(1), crate::Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(1), crate::Value::Int(2), crate::Value::Int(1)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(1), crate::Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(1), crate::Value::Int(2), crate::Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(1), crate::Value::Int(2), crate::Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(1), crate::Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(2), crate::Value::Int(1)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![crate::Value::Int(2)]);
		interpret.tick();
	}
}

