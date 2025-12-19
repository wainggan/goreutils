
use goreutils::args;

struct Config {
	help: bool,
	version: bool,
	output: Option<String>,
	size: (u32, u32),
}
#[allow(clippy::derivable_impls)]
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
Usage: kibt [OPTION]...
Scripting.
  -o, --output [x]  save output image to x (default=output.bmp)
  -d, -size [x] [y] set canvas size to width=x and height=y
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


#[derive(Debug, PartialEq)]
enum TokenError {
	UnknownChar,
	// UnterminatedString,
}

#[derive(Debug, PartialEq)]
enum Kind {
	Eof,
	Error(TokenError),
	Whitespace,
	LParen,
	RParen,
	LBrace,
	RBrace,
	Int,
	Flt,
	Ident,
	// String,
}

#[derive(Debug, PartialEq)]
struct Token<'a> {
	kind: Kind,
	data: &'a str,
}

#[derive(Debug, Clone)]
struct Tokenize<'a> {
	src: &'a str,
	iter: std::str::Chars<'a>,
	offset: usize,
}

impl<'a> Tokenize<'a> {
	fn new(src: &'a str) -> Self {
		Self {
			src,
			iter: src.chars(),
			offset: src.len(),
		}
	}

	fn peek_one(&self) -> Option<char> {
		self.iter.clone().next()
	}

	fn at_end(&self) -> bool {
		self.iter.as_str().is_empty()
	}

	fn token_pos(&self) -> usize {
		self.src.len() - self.offset
	}

	fn token_len(&self) -> usize {
		self.offset - self.iter.as_str().len()
	}

	fn token_reset(&mut self) {
		self.offset = self.iter.as_str().len();
	}

	fn token_emit(&self, kind: Kind) -> Token<'a> {
		let start = self.token_pos();
		let end = start + self.token_len();
		Token {
			kind,
			data: &self.src[start..end],
		}
	}

	fn bump(&mut self) -> Option<char> {
		self.iter.next()
	}
	fn bump_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
		while let Some(c) = self.peek_one() && predicate(c) && !self.at_end() {
			self.bump();
		}
	}

	fn advance(&mut self) -> Token<'a> {
		let Some(c) = self.bump() else {
			return Token {
				kind: Kind::Eof,
				data: "",
			};
		};

		let k = match c {
			'(' => Kind::LParen,
			')' => Kind::RParen,
			'{' => Kind::LBrace,
			'}' => Kind::RBrace,
			c if c.is_whitespace() => {
				self.bump_while(|x| x.is_whitespace());
				Kind::Whitespace
			}
			c if c.is_alphabetic() => {
				self.bump_while(|x| x.is_alphanumeric() || x == '_');
				Kind::Ident
			}
			c if c.is_numeric() => {
				self.bump_while(|x| x.is_numeric());
				if matches!(self.peek_one(), Some(x) if x == '.') {
					self.bump();
					self.bump_while(|x| x.is_numeric());
					Kind::Flt
				} else {
					Kind::Int
				}
			}
			_ => Kind::Error(TokenError::UnknownChar),
		};

		let o = self.token_emit(k);
		self.token_reset();
		o
	}
}

impl<'a> Iterator for Tokenize<'a> {
	type Item = Token<'a>;
	fn next(&mut self) -> Option<Self::Item> {
		while !self.at_end() {
			let token = self.advance();
			if !matches!(token.kind, Kind::Whitespace) {
				return Some(token);
			}
		}
		None
	}
}

mod ops {
	pub const NOP: u8 = 0x00;
	pub const POP: u8 = 0x01;
	pub const GET: u8 = 0x02;
	pub const SET: u8 = 0x03;
	pub const SWAP: u8 = 0x04;
	pub const CALL: u8 = 0x05;
	pub const LIT_INT: u8 = 0x10;
	pub const LIT_FLT: u8 = 0x11;
	pub const LIT_NONE: u8 = 0x12;
}

struct Globals<T> {
	values: Vec<(&'static str, Value)>,
	native: Vec<fn(&mut dyn FnMut() -> Option<Value>, &T) -> Value>,
}
impl<T> Globals<T> {
	fn new(values: Vec<(&'static str, Value)>, native: Vec<fn(&mut dyn FnMut() -> Option<Value>, &T) -> Value>) -> Self {
		Self {
			values,
			native,
		}
	}
}

struct Compile<'a, I: Iterator<Item = Token<'a>>> {
	tokens: std::iter::Peekable<I>,
	env: Vec<(&'a str, u32)>,
	scope_depth: u32,
}
impl<'a, I: Iterator<Item = Token<'a>>> Compile<'a, I> {
	fn new<T>(tokens: I, globals: &Globals<T>) -> Self {
		let env = globals.values.iter().map(|x| (x.0, 0)).collect();
		Self {
			tokens: tokens.peekable(),
			env,
			scope_depth: 0,
		}
	}

	fn check(&mut self, kinds: &[Kind]) -> Option<Token<'a>> {
		let peek = self.tokens.peek()?;
		for k in kinds {
			if *k == peek.kind {
				return self.tokens.next();
			}
		}
		None
	}
	fn crack(&mut self, mut predicate: impl FnMut(&Token<'a>) -> bool) -> Option<Token<'a>> {
		if predicate(self.tokens.peek()?) {
			self.tokens.next()
		} else {
			None
		}
	}

	fn at_end(&mut self) -> bool {
		self.tokens.peek().is_none()
	}

	fn parse(mut self) -> Result<Vec<u8>, String> {
		let mut bin = vec![];
		self.module(&mut bin)?;
		Ok(bin)
	}

	fn module(&mut self, bin: &mut Vec<u8>) -> Result<(), String> {
		self.primary(bin)
	}

	fn block(&mut self, bin: &mut Vec<u8>) -> Result<(), String> {
		self.scope_depth += 1;
		
		while !self.at_end() {
			if self.check(&[Kind::RBrace]).is_some() {
				break;
			}

			// returning here should be fine, so long as no other code tries to handle the error
			// todo: that sucks though

			if self.crack(|x| matches!((&x.kind, x.data), (&Kind::Ident, "let"))).is_some() {
				let name = self.check(&[Kind::Ident]).ok_or_else(|| "missing var name".to_string())?;
				self.primary(bin)?;
				self.env.push((name.data, self.scope_depth));
				bin.push(ops::LIT_NONE);
			}
			else if self.crack(|x| matches!((&x.kind, x.data), (&Kind::Ident, "set"))).is_some() {
				let name = self.check(&[Kind::Ident]).ok_or_else(|| "missing var name".to_string())?;

				self.primary(bin)?;

				let pos = self.env.iter()
					.zip(0..self.env.len())
					.rev()
					.find(|y| y.0.0 == name.data)
					.map(|y| y.1)
					.ok_or_else(|| format!("unknown variable {}", name.data))?;

				bin.push(ops::SET);
				bin.push(pos as u8);

				bin.push(ops::LIT_NONE);
			}
			else {
				self.primary(bin)?;
			}
			bin.push(ops::POP);
		}
		bin.pop(); // lol

		self.scope_depth -= 1;

		while let Some(x) = self.env.last()
		&& x.1 > self.scope_depth {
			bin.push(ops::SWAP);
			bin.push(ops::POP);
			self.env.pop();
		}
		
		Ok(())
	}

	fn list(&mut self, bin: &mut Vec<u8>) -> Result<(), String> {
		self.primary(bin)?;

		let mut count = 0;
		
		while !self.at_end() {
			if self.check(&[Kind::RParen]).is_some() {
				break;
			}
			self.primary(bin)?;

			bin.push(ops::SWAP);

			count += 1;
		}

		bin.push(ops::CALL);
		bin.push(count);

		Ok(())
	}

	fn primary(&mut self, bin: &mut Vec<u8>) -> Result<(), String> {
		if self.check(&[Kind::LParen]).is_some() {
			return self.list(bin);
		}

		if self.check(&[Kind::LBrace]).is_some() {
			return self.block(bin);
		}

		if let Some(x) = self.check(&[Kind::Int]) {
			let a = x.data.parse::<i32>().map_err(|_| "number parse error".to_string())?;
			bin.push(ops::LIT_INT);
			bin.extend_from_slice(&a.to_be_bytes());
			return Ok(());
		}

		if let Some(x) = self.check(&[Kind::Flt]) {
			let a = x.data.parse::<f32>().map_err(|_| "number parse error".to_string())?;
			bin.push(ops::LIT_FLT);
			bin.extend_from_slice(&a.to_be_bytes());
			return Ok(());
		}

		// if let Some(x) = self.check(&[Kind::String]) {
			//return Ok(Value::Str(x.data.to_string()));
		// }

		if let Some(x) = self.check(&[Kind::Ident]) {
			let pos = self.env.iter()
				.zip(0..self.env.len())
				.rev()
				.find(|y| y.0.0 == x.data)
				.map(|y| y.1)
				.ok_or_else(|| format!("unknown variable {}", x.data))?;

			bin.push(ops::GET);
			bin.push(pos as u8);

			return Ok(());
		}

		Err(format!("unexpected token: {:?}", self.tokens.peek()))
	}
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
	None,
	Int(i32),
	Flt(f32),
	// FUCK
	// todo: OPTIMIZE
	List(Vec<Value>),
	Fn(u32),
}

struct Interpret<'a, 'b, 'c, T> {
	pc: usize,
	stack: Vec<Value>,
	native: &'a Vec<fn(&mut dyn FnMut() -> Option<Value>, &T) -> Value>,
	bin: &'c Vec<u8>,
	env: &'b T,
}
impl<'a, 'b, 'c, T> Interpret<'a, 'b, 'c, T> {
	fn new(bin: &'c Vec<u8>, globals: &'a Globals<T>, env: &'b T) -> Self {
		let stack = globals.values.iter().map(|x| x.1.clone()).collect();
		Self {
			bin,
			stack,
			native: &globals.native,
			pc: 0,
			env,
		}
	}

	fn next<const N: usize>(&mut self) -> Option<[u8; N]> {
		let x = self.pc;
		self.pc += N;
		let y = self.pc;
		self.bin.get(x..y)?.try_into().ok()
	}

	fn end(&self) -> bool {
		self.pc >= self.bin.len()
	}

	fn byte(&mut self) -> u8 {
		u8::from_be_bytes(self.next::<1>().unwrap())
	}

	fn word_i32(&mut self) -> i32 {
		i32::from_be_bytes(self.next::<4>().unwrap())
	}
	
	fn word_f32(&mut self) -> f32 {
		f32::from_be_bytes(self.next::<4>().unwrap())
	}

	fn tick(&mut self) {
		if self.end() {
			return;
		}
		let inst = self.byte();
		match inst {
			ops::NOP => {

			}
			ops::POP => {
				self.stack.pop();
			}
			ops::SWAP => {
				let data_0 = self.stack.pop().unwrap();
				let data_1 = self.stack.pop().unwrap();
				self.stack.push(data_0);
				self.stack.push(data_1);
			}
			ops::GET => {
				let offset = self.byte();
				let data = self.stack[offset as usize].clone();
				self.stack.push(data);
			}
			ops::SET => {
				let offset = self.byte();
				self.stack[offset as usize] = self.stack.pop().unwrap();
			}
			ops::LIT_INT => {
				let data = self.word_i32();
				self.stack.push(Value::Int(data));
			}
			ops::LIT_FLT => {
				let data = self.word_f32();
				self.stack.push(Value::Flt(data));
			}
			ops::LIT_NONE => {
				self.stack.push(Value::None);
			}
			ops::CALL => {
				let cmd = self.stack.pop().unwrap();
				let mut count = self.byte();
				match cmd {
					Value::Fn(x) => {
						let f = self.native[x as usize];
						let out = f(&mut || {
							if count > 0 {
								count -= 1;
								Some(self.stack.remove(self.stack.len() - count as usize - 1))
							} else {
								None
							}
						}, self.env);
						while count > 0 {
							count -= 1;
						}
						self.stack.push(out);
					}
					_ => {
						println!("{:?} {:?}", cmd, self.stack);
						panic!("whyyyy");
					},
				}
			}
			_ => unimplemented!(),
		}
	}
}




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

	struct Env {
		uv: (f32, f32),
		px: (u32, u32),
		size: (u32, u32),
	}

	let globals = Globals::<Env>::new({
		let mut vec = vec![];

		vec.push(("pi", Value::Flt(core::f32::consts::PI)));
		vec.push(("none", Value::None));

		vec.push(("print", Value::Fn(0)));
		vec.push(("int", Value::Fn(1)));
		vec.push(("flt", Value::Fn(2)));
		vec.push(("add", Value::Fn(3)));
		vec.push(("sub", Value::Fn(4)));
		vec.push(("mul", Value::Fn(5)));
		vec.push(("div", Value::Fn(6)));
		vec.push(("list", Value::Fn(7)));
		vec.push(("uv_x", Value::Fn(8)));
		vec.push(("uv_y", Value::Fn(9)));
		vec.push(("px_x", Value::Fn(10)));
		vec.push(("px_y", Value::Fn(11)));
		vec.push(("width", Value::Fn(12)));
		vec.push(("height", Value::Fn(13)));

		vec
	}, vec![
		|p, _| {
			while let Some(value) = p() {
				println!("{:?}", value);
			}
			Value::None
		},
		|p, _| {
			match p().unwrap_or(Value::None) {
				Value::Int(x) => Value::Int(x),
				Value::Flt(x) => Value::Int(x as i32),
				_ => Value::Int(0),
			}
		},
		|p, _| {
			match p().unwrap_or(Value::None) {
				Value::Flt(x) => Value::Flt(x),
				Value::Int(x) => Value::Flt(x as f32),
				_ => Value::Int(0),
			}
		},
		|p, _| {
			let mut acc = Value::None;
			loop {
				let n = p().unwrap_or(Value::None);
				if matches!(n, Value::None) {
					break;
				}
				acc = match (acc, n) {
					(Value::None, y) => y,
					(Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_add(y)),
					(Value::Flt(x), Value::Flt(y)) => Value::Flt(x + y),
					_ => Value::None,
				}
			}
			acc
		},
		|p, _| {
			let mut acc = Value::None;
			loop {
				let n = p().unwrap_or(Value::None);
				if matches!(n, Value::None) {
					break;
				}
				acc = match (acc, n) {
					(Value::None, y) => y,
					(Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_sub(y)),
					(Value::Flt(x), Value::Flt(y)) => Value::Flt(x - y),
					_ => Value::None,
				}
			}
			acc
		},
		|p, _| {
			let mut acc = Value::None;
			loop {
				let n = p().unwrap_or(Value::None);
				if matches!(n, Value::None) {
					break;
				}
				acc = match (acc, n) {
					(Value::None, y) => y,
					(Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_mul(y)),
					(Value::Flt(x), Value::Flt(y)) => Value::Flt(x * y),
					_ => Value::None,
				}
			}
			acc
		},
		|p, _| {
			let mut acc = Value::None;
			loop {
				let n = p().unwrap_or(Value::None);
				if matches!(n, Value::None) {
					break;
				}
				acc = match (acc, n) {
					(Value::None, y) => y,
					(Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_div(y)),
					(Value::Flt(x), Value::Flt(y)) => Value::Flt(x / y),
					_ => Value::None,
				}
			}
			acc
		},
		|p, _| {
			let mut vec = vec![];
			while let Some(value) = p() {
				vec.push(value);
			}
			Value::List(vec)
		},
		|_, env| {
			Value::Flt(env.uv.0)
		},
		|_, env| {
			Value::Flt(env.uv.1)
		},
		|_, env| {
			Value::Int(env.px.0 as i32)
		},
		|_, env| {
			Value::Int(env.px.1 as i32)
		},
		|_, env| {
			Value::Int(env.size.0 as i32)
		},
		|_, env| {
			Value::Int(env.size.1 as i32)
		},
	]);

	let mut canvas = vec![(0u8, 0u8, 0u8); (config.size.0 * config.size.1) as usize];

	for src in value {
		let tokens = crate::Tokenize::new(&src).collect::<Vec<_>>();

		let bin = crate::Compile::new(tokens.into_iter(), &globals).parse().unwrap();

		for (i, poke) in canvas.iter_mut().enumerate() {
			let px = (i as u32 % config.size.0, i as u32 / config.size.0);
			let env = Env {
				uv: (px.0 as f32 / config.size.0 as f32, px.1 as f32 / config.size.1 as f32),
				px: (px.0, px.1),
				size: (config.size.0, config.size.1),
			};

			let mut vm = crate::Interpret::new(&bin, &globals, &env);
			while !vm.end() {
				vm.tick();
			}

			let out = vm.stack.pop().unwrap_or(Value::None);
			
			let map = |x: Value| match x {
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
				for _ in 0..fold {
					buffer.push(0);
				}
			}
		}
	}

	std::fs::write(
		config.output.as_deref()
			.unwrap_or("output.bmp"),
		&buffer
	).unwrap();
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

		let globals = crate::Globals::new(vec![], vec![]);

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

		let globals = crate::Globals::new(vec![], vec![]);

		let bin = crate::Compile::new(tokens.into_iter(), &globals).parse().unwrap();

		assert_eq!(&bin, &[
			crate::ops::LIT_INT,
			0, 0, 0, 1,
			crate::ops::LIT_INT,
			0, 0, 0, 2,
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
		assert_eq!(interpret.stack, vec![crate::Value::Int(1)]);
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

