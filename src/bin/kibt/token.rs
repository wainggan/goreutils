
use crate::types::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct Tokenize<'a> {
	src: &'a str,
	iter: std::str::Chars<'a>,
	offset: usize,
}

impl<'a> Tokenize<'a> {
	pub fn new(src: &'a str) -> Self {
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

	fn token_src(&self) -> &'a str {
		let start = self.token_pos();
		let end = start + self.token_len();
		&self.src[start..end]
	}

	fn token_emit(&self, kind: TokenKind, src: &'a str) -> Token<'a> {
		Token::new(kind, src)
	}

	fn bump(&mut self) -> Option<char> {
		self.iter.next()
	}

	fn bump_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
		while let Some(c) = self.peek_one() && predicate(c) && !self.at_end() {
			self.bump();
		}
	}

	pub fn advance(&mut self) -> Token<'a> {
		let Some(c) = self.bump() else {
			return Token::new(TokenKind::Eof, "");
		};

		let k = match c {
			'(' => TokenKind::LParen,
			')' => TokenKind::RParen,
			'{' => TokenKind::LBrace,
			'}' => TokenKind::RBrace,
			c if c.is_whitespace() => {
				self.bump_while(|x| x.is_whitespace());
				TokenKind::Whitespace
			}
			c if c.is_alphabetic() => {
				self.bump_while(|x| x.is_alphanumeric() || x == '_');
				match self.token_src() {
					"let" => TokenKind::Let,
					"set" => TokenKind::Set,
					"if" => TokenKind::If,
					"else" => TokenKind::Else,
					"loop" => TokenKind::Loop,
					"break" => TokenKind::Break,
					"continue" => TokenKind::Continue,
					"none" => TokenKind::None,
					_ => TokenKind::Ident,
				}
			}
			c if c.is_numeric() || c == '-' => {
				self.bump_while(|x| x.is_numeric());
				if matches!(self.peek_one(), Some(x) if x == '.') {
					self.bump();
					self.bump_while(|x| x.is_numeric());
					TokenKind::Flt
				} else {
					TokenKind::Int
				}
			}
			_ => TokenKind::Error(crate::types::TokenError::UnknownChar),
		};

		let o = self.token_emit(k, self.token_src());
		self.token_reset();
		o
	}
}

impl<'a> Iterator for Tokenize<'a> {
	type Item = Token<'a>;
	fn next(&mut self) -> Option<Self::Item> {
		while !self.at_end() {
			let token = self.advance();
			if !matches!(token.kind(), TokenKind::Whitespace) {
				return Some(token);
			}
		}
		None
	}
}

#[cfg(test)]
mod test {
    use crate::{token::Tokenize, types::{Token, TokenKind}};

	#[test]
	fn test_tokens() {
		let src = "(0 0.0 test) (0)";
		let vec = Tokenize::new(src).collect::<Vec<_>>();
		assert_eq!(
			&vec,
			&[
				Token::new(TokenKind::LParen, "("),
				Token::new(TokenKind::Int, "0"),
				Token::new(TokenKind::Flt, "0.0"),
				Token::new(TokenKind::Ident, "test"),
				Token::new(TokenKind::RParen, ")"),
				Token::new(TokenKind::LParen, "("),
				Token::new(TokenKind::Int, "0"),
				Token::new(TokenKind::RParen, ")"),
			],
		);
	}
}

