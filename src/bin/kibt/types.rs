
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


const VALUE_TAG_SIGNAL: u64 =
	0xfff4_0000_0000_0000;
const VALUE_TAG_QUIET: u64 =
	0xfff8_0000_0000_0000;
const VALUE_TAG_MASK: u64 =
	0xffff_0000_0000_0000;

const VALUE_TYPE_MASK: u64 =
	0x0000_ff00_0000_0000;
const VALUE_TYPE_NONE: u64 =
	0x0000_0000_0000_0000;
const VALUE_TYPE_BOOL: u64 =
	0x0000_0100_0000_0000;
const VALUE_TYPE_INT: u64 =
	0x0000_0200_0000_0000;
const VALUE_TYPE_STR_LIT: u64 =
	0x0000_0300_0000_0000;
const VALUE_TYPE_STR_OWN: u64 =
	0x0000_0400_0000_0000;
const VALUE_TYPE_LIST: u64 =
	0x0000_0500_0000_0000;
const VALUE_TYPE_FN_NATIVE: u64 =
	0x0000_0600_0000_0000;

const VALUE_INDEX_MASK: u64 =
	0x0000_0000_ffff_ffff;

#[derive(Debug)]
pub struct Val(f64);

impl Val {
	pub fn new_f64(value: f64) -> Val {
		Val(value)
	}

	pub fn get_f64(&self) -> f64 {
		self.0
	}

	pub fn get_u64(&self) -> u64 {
		self.0.to_bits()
	}

	pub fn mask_tag(&self) -> u64 {
		self.0.to_bits() & VALUE_TAG_MASK
	}

	pub fn mask_type(&self) -> u64 {
		self.0.to_bits() & VALUE_TYPE_MASK
	}

	pub fn mask_index(&self) -> u32 {
		(self.0.to_bits() & VALUE_INDEX_MASK) as u32
	}

	pub fn tagged(&self) -> Tagged {
		if self.mask_tag() == VALUE_TAG_SIGNAL {
			return Tagged::Flt(self.0);
		}

		let tt = self.mask_type();
		let dd = self.mask_index();

		match tt {
			VALUE_TYPE_NONE => Tagged::None,
			VALUE_TYPE_BOOL => Tagged::Bool(dd != 0),
			VALUE_TYPE_INT => Tagged::Int(dd.cast_signed()),
			_ => panic!("unknown tag: {:x}", dd),
		}
	}
}

#[derive(Debug)]
pub enum Tagged {
	None,
	Bool(bool),
	Int(i32),
	Flt(f64),
}

#[derive(Debug, Clone)]
pub struct ListIndex(pub u32);

#[derive(Debug)]
pub enum Value {
	None,
	Bool(bool),
	Int(i32),
	Flt(f32),
	Str(String),
	// FUCK
	// todo: OPTIMIZE
	List(ListIndex),
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

