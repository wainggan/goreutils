use crate::{compile::ops, types::{Library, NativeFnIndex, Tagged, Value}};

pub struct Interpret<'a, 'b, 'c: 'b, Env> {
	pc: usize,
	stack: Vec<Value>,
	bin: &'a Vec<u8>,
	env: &'b Env,
	natives: Library<'c, Env>,
}

impl<'a, 'b, 'c, Env> Interpret<'a, 'b, 'c, Env> {
	pub fn new(bin: &'a Vec<u8>, natives: Library<'c, Env>, env: &'b Env) -> Self {
		let stack = natives
			.iter()
			.enumerate()
			.map(|(i, _)|
				Value::new_tagged(Tagged::NativeFn(NativeFnIndex(i as u32)))
			)
			.collect();
		Self {
			bin,
			stack,
			pc: 0,
			env,
			natives,
		}
	}

	fn next<const N: usize>(&mut self) -> Option<[u8; N]> {
		let x = self.pc;
		self.pc += N;
		let y = self.pc;
		self.bin.get(x..y)?.try_into().ok()
	}

	pub fn end(&self) -> bool {
		self.pc >= self.bin.len()
	}

	fn byte(&mut self) -> u8 {
		u8::from_be_bytes(self.next::<1>().unwrap())
	}

	fn word_u16(&mut self) -> u16 {
		u16::from_be_bytes(self.next::<2>().unwrap())
	}

	fn word_i32(&mut self) -> i32 {
		i32::from_be_bytes(self.next::<4>().unwrap())
	}

	fn word_f64(&mut self) -> f64 {
		f64::from_be_bytes(self.next::<8>().unwrap())
	}

	fn slice(&mut self, len: usize) -> &[u8] {
		let pc = self.pc;
		self.pc += len;
		&self.bin[pc..pc + len]
	}

	pub fn pop(&mut self) -> Option<Value> {
		self.stack.pop()
	}

	pub fn tick(&mut self) -> Result<(), String> {
		if self.end() {
			return Ok(());
		}

		let inst = self.byte();
		match inst {
			ops::NOP => {}

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

			ops::JUMP => {
				let offset = self.word_i32().cast_unsigned();
				let condition = self.stack.pop().map(|x| x.get_tagged());
				let Some(Tagged::Bool(value)) = condition else {
					panic!();
				};
				if value {
					self.pc = offset as usize;
				}
			}

			ops::LIT_INT => {
				let data = self.word_i32();
				self.stack.push(Value::new_tagged(Tagged::Int(data)));
			}

			ops::LIT_FLT => {
				let data = self.word_f64();
				self.stack.push(Value::new_tagged(Tagged::Flt(data)));
			}

			ops::LIT_STR => {
				todo!();
				// let len = self.word_u16();
				// let data = self.slice(len.into());
				// let s = str::from_utf8(data)
				// 	.unwrap()
				// 	.to_string();
				// self.stack.push(Value::Str(s));
			}

			ops::LIT_NONE => {
				self.stack.push(Value::new_tagged(Tagged::None));
			}

			ops::LIT_TRUE => {
				self.stack.push(Value::new_tagged(Tagged::Bool(true)));
			}

			ops::LIT_FALSE => {
				self.stack.push(Value::new_tagged(Tagged::Bool(false)));
			}

			ops::CALL => {
				let cmd = self.stack.pop().unwrap().get_tagged();
				let mut count = self.byte();
				match cmd {
					Tagged::NativeFn(x) => {
						let y = self.natives[x.0 as usize].1;

						let out = y(&mut || {
							if count > 0 {
								count -= 1;
								Some(self.stack.remove(self.stack.len() - count as usize - 1).get_tagged())
							}
							else {
								None
							}
						}, self.env);

						while count > 0 {
							self.stack.pop();
							count -= 1;
						}

						self.stack.push(Value::new_tagged(out));
					}
					_ => {
						println!("{:?} {:?}", cmd, self.stack);
						panic!("whyyyy");
					},
				}
			}
			_ => unimplemented!(),
		}

		Ok(())
	}
}


#[cfg(test)]
mod test {
    use crate::{compile::{Compile, ops}, interpret::Interpret, token::Tokenize, types::{Tagged, Value}};

	#[test]
	fn test_interpret_0() {
		let src = "{1 0.2}";

		let tokens = Tokenize::new(src).collect::<Vec<_>>();
		println!("{:?}", tokens);

		let globals = [];

		let bin = Compile::new(tokens.into_iter(), &globals).parse().unwrap();

		assert_eq!(&bin, &[
			ops::LIT_INT,
			0, 0, 0, 1,
			ops::POP,
			ops::LIT_FLT,
			63, 201, 153, 153, 153, 153, 153, 154,
		]);

		let mut interpret = Interpret::new(&bin, &globals, &());

		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(1))]);
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![]);
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Flt(0.2))]);
		_ = interpret.tick();
	}

	#[test]
	fn test_interpret_1() {
		let src = "{ let a 1 let b 2 a b }";

		let tokens = Tokenize::new(src).collect::<Vec<_>>();
		println!("{:?}", tokens);

		let globals = [];

		let bin = Compile::new(tokens.into_iter(), &globals).parse().unwrap();

		assert_eq!(&bin, &[
			ops::LIT_INT,
			0, 0, 0, 1,
			ops::LIT_NONE,
			ops::POP,

			ops::LIT_INT,
			0, 0, 0, 2,
			ops::LIT_NONE,
			ops::POP,

			ops::GET,
			0,
			ops::POP,

			ops::GET,
			1,
			ops::SWAP,
			ops::POP,

			ops::SWAP,
			ops::POP,
		]);

		let mut interpret = Interpret::new(&bin, &globals, &());

		_ = interpret.tick();
		_ = interpret.tick();
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(1))]);
		_ = interpret.tick();
		_ = interpret.tick();
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(1)), Value::new_tagged(Tagged::Int(2))]);
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(1)), Value::new_tagged(Tagged::Int(2)), Value::new_tagged(Tagged::Int(1))]);
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(1)), Value::new_tagged(Tagged::Int(2))]);
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(1)), Value::new_tagged(Tagged::Int(2)), Value::new_tagged(Tagged::Int(2))]);
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(1)), Value::new_tagged(Tagged::Int(2)), Value::new_tagged(Tagged::Int(2))]);
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(1)), Value::new_tagged(Tagged::Int(2))]);
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(2)), Value::new_tagged(Tagged::Int(1))]);
		_ = interpret.tick();
		assert_eq!(interpret.stack, vec![Value::new_tagged(Tagged::Int(2))]);
		_ = interpret.tick();
	}
}
