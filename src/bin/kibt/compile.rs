use crate::types::{Library, Token, TokenKind};

pub mod ops {
	/// do nothing
	pub const NOP: u8 = 0x00;
	/// consume top value in stack
	pub const POP: u8 = 0x01;
	/// consume 
	pub const GET: u8 = 0x02;
	pub const SET: u8 = 0x03;

	pub const SWAP: u8 = 0x04;
	
	/// - operand
	///     - `target`: `[u8; 4]`
	/// - consume
	///     1. `condition`
	/// 
	/// jump to an address.
	/// 
	/// - if `condition` is a `Value::Int`
	///     - if it is `!= 0`, then
	///       this does nothing.
	///     - else, this sets the `pc` to the
	///       address in `target`.
	/// - else, this traps.
	/// 
	pub const JUMP: u8 = 0x05;
	
	/// - operand
	///     - `arg_count`: `[u8; 1]`
	/// - consume
	///     1. `arg`
	/// 
	/// attempt to call `arg` as a function.
	pub const CALL: u8 = 0x06;
	
	/// - operand
	///     - `x`: `[u8; 4]`
	/// 
	/// pushes `x` onto the stack as a `Value::Int`
	pub const LIT_INT: u8 = 0x10;

	/// - operand
	///     - `x`: `[u8; 4]`
	/// 
	/// pushes `x` onto the stack as a `Value::Flt`
	pub const LIT_FLT: u8 = 0x11;

	/// pushes a `Value::None` onto the stack
	pub const LIT_NONE: u8 = 0x12;
}

pub struct Compile<'a, I: Iterator<Item = Token<'a>>> {
	tokens: std::iter::Peekable<I>,
	env: Vec<(&'a str, u32)>,
	scope_depth: u32,
}
impl<'a, I: Iterator<Item = Token<'a>>> Compile<'a, I> {
	pub fn new<Env: ?Sized>(tokens: I, globals: Library<Env>) -> Self {
		let env = globals.iter()
			.map(|x| (x.0, 0))
			.collect();
		Self {
			tokens: tokens.peekable(),
			env,
			scope_depth: 0,
		}
	}

	fn check(&mut self, kinds: &[TokenKind]) -> Option<Token<'a>> {
		let peek = self.tokens.peek()?;
		for k in kinds {
			if *k == peek.kind() {
				return self.tokens.next();
			}
		}
		None
	}

	#[expect(unused, reason = "might need later")]
	fn crack(&mut self, mut predicate: impl FnMut(&Token<'a>) -> bool) -> Option<Token<'a>> {
		if predicate(self.tokens.peek()?) {
			self.tokens.next()
		}
		else {
			None
		}
	}

	fn at_end(&mut self) -> bool {
		self.tokens.peek().is_none()
	}

	pub fn parse(mut self) -> Result<Vec<u8>, String> {
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
			if self.check(&[TokenKind::RBrace]).is_some() {
				break;
			}

			// returning here should be fine, so long as no other code tries to handle the error
			// todo: that sucks though

			if self.check(&[TokenKind::Let]).is_some() {
				let name = self.check(&[TokenKind::Ident]).ok_or_else(|| "missing var name".to_string())?;
				self.primary(bin)?;
				self.env.push((name.src(), self.scope_depth));
				bin.push(ops::LIT_NONE);
			}
			else if self.check(&[TokenKind::Set]).is_some() {
				let name = self.check(&[TokenKind::Ident]).ok_or_else(|| "missing var name".to_string())?;

				self.primary(bin)?;

				let pos = self.env.iter()
					.zip(0..self.env.len())
					.rev()
					.find(|y| y.0.0 == name.src())
					.map(|y| y.1)
					.ok_or_else(|| format!("unknown variable {}", name.src()))?;

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
			if self.check(&[TokenKind::RParen]).is_some() {
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
		if self.check(&[TokenKind::LParen]).is_some() {
			return self.list(bin);
		}

		if self.check(&[TokenKind::LBrace]).is_some() {
			return self.block(bin);
		}

		if self.check(&[TokenKind::If]).is_some() {
			// compile condition
			self.primary(bin)?;

			// use the result of the condition to jump if 'false'
			bin.push(ops::JUMP);

			// jump takes an operand of where to jump to.
			// we don't know where to jump yet, so we keep an index
			// to the operand to fill in later.
			// we want to jump to the 'else' branch.
			let offset_else = bin.len();
			bin.extend_from_slice(&0u32.to_be_bytes());

			// compile 'then' branch
			self.primary(bin)?;

			// unconditional jump to the end
			bin.push(ops::LIT_INT);
			bin.extend_from_slice(&0u32.to_be_bytes());
			bin.push(ops::JUMP);

			// fill in later again
			// this jump wants to reach the end.
			let offset_complete = bin.len();
			bin.extend_from_slice(&0u32.to_be_bytes());

			// parse 'else' keyword
			self.check(&[TokenKind::Else]).ok_or_else(|| "missing 'else' branch".to_string())?;

			// we are now at the 'else' branch, which
			// `offset_else` wants to target.
			let target_else = bin.len() as u32;

			// compile 'else' branch
			self.primary(bin)?;

			// we are now at the end, which `offset_complete`
			// wants to target.
			let target_complete = bin.len() as u32;

			bin[offset_else..offset_else + 4].swap_with_slice(&mut target_else.to_be_bytes());
			bin[offset_complete..offset_complete + 4].swap_with_slice(&mut target_complete.to_be_bytes());

			return Ok(());
		}

		if self.check(&[TokenKind::None]).is_some() {
			bin.push(ops::LIT_NONE);
			return Ok(());
		}

		if let Some(x) = self.check(&[TokenKind::Int]) {
			let a = x.src().parse::<i32>().map_err(|_| "number parse error".to_string())?;
			bin.push(ops::LIT_INT);
			bin.extend_from_slice(&a.to_be_bytes());
			return Ok(());
		}

		if let Some(x) = self.check(&[TokenKind::Flt]) {
			let a = x.src().parse::<f32>().map_err(|_| "number parse error".to_string())?;
			bin.push(ops::LIT_FLT);
			bin.extend_from_slice(&a.to_be_bytes());
			return Ok(());
		}

		if let Some(x) = self.check(&[TokenKind::Ident]) {
			let pos = self.env.iter()
				.zip(0..self.env.len())
				.rev()
				.find(|y| y.0.0 == x.src())
				.map(|y| y.1)
				.ok_or_else(|| format!("unknown variable {}", x.src()))?;

			bin.push(ops::GET);
			bin.push(pos as u8);

			return Ok(());
		}

		Err(format!("unexpected token: {:?}", self.tokens.peek()))
	}
}

