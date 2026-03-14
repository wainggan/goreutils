
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

pub type NativeFn<E> = for<'a> fn(&'a mut (dyn FnMut() -> Option<Tagged> + 'a), &'_ E) -> Tagged;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct Value(f64);

impl Value {
	pub fn new_f64(value: f64) -> Value {
		Value(value)
	}

	pub fn new_tagged(tag: Tagged) -> Self {
		let value = match tag {
			Tagged::None => VALUE_TAG_SIGNAL | VALUE_TYPE_NONE,
			Tagged::Bool(x) => VALUE_TAG_SIGNAL | VALUE_TYPE_BOOL | x as u64,
			Tagged::Int(x) => VALUE_TAG_SIGNAL | VALUE_TYPE_INT | x.cast_unsigned() as u64,
			Tagged::Flt(x) => x.to_bits(),
			Tagged::List(x) => VALUE_TAG_SIGNAL | VALUE_TYPE_LIST | x.0 as u64,
			Tagged::NativeFn(x) => VALUE_TAG_SIGNAL | VALUE_TYPE_FN_NATIVE | x.0 as u64,
		};
		Value(f64::from_bits(value))
	}

	pub fn get_f64(&self) -> f64 {
		self.0
	}

	pub fn get_u64(&self) -> u64 {
		self.0.to_bits()
	}

	/// get tags. will either be [[VALUE_TAG_SIGNAL]] or [[VALUE_TAG_QUIET]].
	pub fn mask_tag(&self) -> u64 {
		self.0.to_bits() & VALUE_TAG_MASK
	}

	/// get type.
	pub fn mask_type(&self) -> u64 {
		self.0.to_bits() & VALUE_TYPE_MASK
	}

	/// retrieve data.
	pub fn mask_index(&self) -> u32 {
		(self.0.to_bits() & VALUE_INDEX_MASK) as u32
	}

	pub fn get_tagged(&self) -> Tagged {
		if self.mask_tag() != VALUE_TAG_SIGNAL {
			return Tagged::Flt(self.0);
		}

		let tt = self.mask_type();
		let dd = self.mask_index();

		match tt {
			VALUE_TYPE_NONE => Tagged::None,
			VALUE_TYPE_BOOL => Tagged::Bool(dd != 0),
			VALUE_TYPE_INT => Tagged::Int(dd.cast_signed()),
			VALUE_TYPE_LIST => Tagged::List(ListIndex(dd)),
			VALUE_TYPE_FN_NATIVE => Tagged::NativeFn(NativeFnIndex(dd)),
			_ => panic!("unknown tag: {:x} {:x}", tt, dd),
		}
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
    	self.get_tagged() == other.get_tagged()
	}
}

#[derive(Debug, PartialEq)]
pub enum Tagged {
	None,
	Bool(bool),
	Int(i32),
	Flt(f64),
	List(ListIndex),
	NativeFn(NativeFnIndex),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListIndex(pub u32);

pub type Library<'a, E> = &'a [LibraryEntry<E>];

pub type LibraryEntry<E> = (&'static str, NativeFn<E>);

#[cfg(test)]
mod test {
    use crate::types::{Tagged, Value};

	#[test]
	fn test_tags() {
		assert_eq!(Value::new_tagged(Tagged::None).get_tagged(), Tagged::None);

		assert_eq!(Value::new_tagged(Tagged::Bool(true)).get_tagged(), Tagged::Bool(true));
		assert_eq!(Value::new_tagged(Tagged::Bool(false)).get_tagged(), Tagged::Bool(false));

		assert_eq!(Value::new_tagged(Tagged::Int(0)).get_tagged(), Tagged::Int(0));
		assert_eq!(Value::new_tagged(Tagged::Int(i32::MIN)).get_tagged(), Tagged::Int(i32::MIN));
		assert_eq!(Value::new_tagged(Tagged::Int(i32::MAX)).get_tagged(), Tagged::Int(i32::MAX));

		assert_eq!(Value::new_tagged(Tagged::Flt(0.0)).get_tagged(), Tagged::Flt(0.0));
		assert_eq!(Value::new_tagged(Tagged::Flt(0.0f64.next_down())).get_tagged(), Tagged::Flt(0.0f64.next_down()));
		assert_eq!(Value::new_tagged(Tagged::Flt(0.0f64.next_up())).get_tagged(), Tagged::Flt(0.0f64.next_up()));
		assert_eq!(Value::new_tagged(Tagged::Flt(f64::INFINITY)).get_tagged(), Tagged::Flt(f64::INFINITY));
		assert_eq!(Value::new_tagged(Tagged::Flt(f64::NEG_INFINITY)).get_tagged(), Tagged::Flt(f64::NEG_INFINITY));
	}
}
