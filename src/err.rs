use std::{fmt, io, error};


pub type JsefResult<T = ()> = Result<T, JsefErr>;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsefErr {
	pub err: JsefErrType,
	
	#[cfg(not(feature = "no-line-col"))]
	pub line: usize,
	
	#[cfg(not(feature = "no-line-col"))]
	pub col: usize,
}

impl JsefErr {
	#[cfg(not(feature = "no-line-col"))]
	pub const fn new(err: JsefErrType, line: usize, col: usize) -> Self {
		Self {err, line, col}
	}
	
	#[cfg(feature = "no-line-col")]
	pub const fn new(err: JsefErrType) -> Self {
		Self {err}
	}
}

impl fmt::Display for JsefErr {
	#[cfg(not(feature = "no-line-col"))]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "JSeF error at line {}, col {}: {}", self.line, self.col, self.err)
	}
	
	#[cfg(feature = "no-line-col")]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "JSeF error: {}", self.err)
	}
}

impl error::Error for JsefErr {}

impl From<JsefErr> for io::Error {
	fn from(err: JsefErr) -> Self {
		Self::other(err)
	}
}


#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsefErrType {
	Unexpected(Option<char>),
	Mismatch(char, Option<char>),
	NotEof(char),
	MaxDepth,
}

impl fmt::Display for JsefErrType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Unexpected(None)     => write!(f, "unexpected EOF"),
			Self::Unexpected(Some(c))  => write!(f, "unexpected '{c}'"),
			Self::Mismatch(e, None)    => write!(f, "expected '{e}', got EOF"),
			Self::Mismatch(e, Some(g)) => write!(f, "expected '{e}', got '{g}'"),
			Self::NotEof(c)            => write!(f, "expected EOF, got '{c}'"),
			Self::MaxDepth             => write!(f, "maximum nesting depth exceeded"),
		}
	}
}

impl error::Error for JsefErrType {}
