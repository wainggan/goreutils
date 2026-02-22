
use crate::{compile::ops, types::{Library, NativeFnIndex, Value}};

pub struct Interpret<'a, Env> {
	pc: usize,
	stack: Vec<Value>,
	bin: &'a Vec<u8>,
	env: &'a Env,
	natives: Library<'a, Env>,
}

impl<'a, Env> Interpret<'a, Env> {
	pub fn new(bin: &'a Vec<u8>, natives: Library<'a, Env>, env: &'a Env) -> Self {
		let stack = natives
			.iter()
			.enumerate()
			.map(|(i, _)|
				Value::NativeFn(NativeFnIndex(i as u32))
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

	fn word_i32(&mut self) -> i32 {
		i32::from_be_bytes(self.next::<4>().unwrap())
	}
	
	fn word_f32(&mut self) -> f32 {
		f32::from_be_bytes(self.next::<4>().unwrap())
	}

	pub fn pop(&mut self) -> Option<Value> {
		self.stack.pop()
	}

	pub fn tick(&mut self) {
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
			ops::JUMP => {
				let offset = self.word_i32().cast_unsigned();
				let condition = self.stack.pop();
				let value = match condition {
					Some(Value::Int(x)) => x == 0,
					_ => panic!(),
				};
				if value {
					self.pc = offset as usize;
				}
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
					Value::NativeFn(x) => {
						let y = self.natives[x.0 as usize].1;

						let out = y(&mut || {
							if count > 0 {
								count -= 1;
								Some(self.stack.remove(self.stack.len() - count as usize - 1))
							}
							else {
								None
							}
						}, self.env);

						while count > 0 {
							self.stack.pop();
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


#[cfg(test)]
mod test {
    use crate::{compile::{Compile, ops}, interpret::Interpret, token::Tokenize, types::Value};

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
			62, 76, 204, 205,
		]);

		let mut interpret = Interpret::new(&bin, &globals, &());

		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(1)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Flt(0.2)]);
		interpret.tick();
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

		interpret.tick();
		interpret.tick();
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(1)]);
		interpret.tick();
		interpret.tick();
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(1), Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(1), Value::Int(2), Value::Int(1)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(1), Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(1), Value::Int(2), Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(1), Value::Int(2), Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(1), Value::Int(2)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(2), Value::Int(1)]);
		interpret.tick();
		assert_eq!(interpret.stack, vec![Value::Int(2)]);
		interpret.tick();
	}
}

