use std::{fmt, error};


pub type JsefResult<T = ()> = Result<T, JsefErr>;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsefErr {
	pub err: JsefErrType,
	pub line: usize,
	pub col: usize,
}

impl JsefErr {
	pub const fn new(err: JsefErrType, line: usize, col: usize) -> Self {
		Self {err, line, col}
	}
}

impl fmt::Display for JsefErr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "JSeF error at line {}, col {}: {}", self.line, self.col, self.err)
	}
}

impl error::Error for JsefErr {}


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
			Self::Unexpected(None) => write!(f, "unexpected EOF"),
			Self::Unexpected(Some(c)) => write!(f, "unexpected '{c}'"),
			Self::Mismatch(c, None) => write!(f, "expected '{c}', got EOF"),
			Self::Mismatch(c, Some(e)) => write!(f, "expected '{c}', got '{e}'"),
			Self::NotEof(c) => write!(f, "expected EOF, got '{c}'"),
			Self::MaxDepth => write!(f, "maximum nesting depth exceeded"),
		}
	}
}

impl error::Error for JsefErrType {}
