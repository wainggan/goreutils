
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenError {
	UnknownChar,
	// UnterminatedString,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
	Eof,
	Error(TokenError),
	Whitespace,
	
	LParen,
	RParen,
	LBrace,
	RBrace,

	Add,
	Sub,
	Mul,
	Div,
	Rem,

	Let,
	Set,
	If,
	Else,
	Loop,
	Break,
	Continue,
	
	None,
	True,
	False,
	Int,
	Flt,
	Ident,
	Str,
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
	kind: TokenKind,
	data: &'a str,
}

impl<'a> Token<'a> {
	pub fn new(kind: TokenKind, src: &'a str) -> Self {
		Self {
			kind,
			data: src,
		}
	}

	pub fn kind(&self) -> TokenKind {
		self.kind
	}

	pub fn src(&self) -> &'a str {
		self.data
	}
}

pub type NativeFn<E> = for<'a> fn(&'a mut (dyn FnMut() -> Option<Value> + 'a), &'_ E) -> Value;

#[derive(Debug, Clone)]
pub struct NativeFnIndex(pub u32);

#[derive(Debug)]
pub enum Value {
	None,
	Bool(bool),
	Int(i32),
	Flt(f32),
	Str(String),
	// FUCK
	// todo: OPTIMIZE
	List(Vec<Value>),
	#[expect(unused, reason = "will use later")]
	Fn(u32),
	NativeFn(NativeFnIndex),
}

impl Clone for Value {
	fn clone(&self) -> Self {
		match self {
			Value::None => Value::None,
			Value::Bool(x) => Value::Bool(*x),
			Value::Int(x) => Value::Int(*x),
			Value::Flt(x) => Value::Flt(*x),
			Value::Str(x) => Value::Str(x.clone()),
			Value::List(x) => Value::List(x.clone()),
			Value::Fn(x) => Value::Fn(*x),
			Value::NativeFn(x) => Value::NativeFn(x.clone()),
		}
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Value::None, Value::None) => true,
			(Value::Bool(x), Value::Bool(y)) => x == y,
			(Value::Int(x), Value::Int(y)) => x == y,
			(Value::Flt(x), Value::Flt(y)) => x == y,
			(Value::Str(x), Value::Str(y)) => x == y,
			(Value::List(x), Value::List(y)) => x == y,
			(Value::Fn(_), Value::Fn(_)) => false,
			(Value::NativeFn(_), Value::NativeFn(_)) => false,
			_ => false,
		}
	}
}

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Value::None => write!(f, "none"),
			Value::Bool(x) => write!(f, "{}", x),
			Value::Int(x) => write!(f, "{}", x),
			Value::Flt(x) => write!(f, "{}", x),
			Value::Str(x) => write!(f, "{}", x),
			Value::List(x) => {
				write!(f, "[ ")?;
				for y in x.iter() {
					write!(f, "{} ", y)?;
				}
				write!(f, "]")
			},
			Value::Fn(_) => write!(f, "<function>"),
			Value::NativeFn(_) => write!(f, "<native function>"),
		}
	}
}

pub type Library<'a, E> = &'a [LibraryEntry<E>];

pub type LibraryEntry<E> = (&'static str, NativeFn<E>);

