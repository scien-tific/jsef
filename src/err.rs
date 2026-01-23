use std::{fmt, error};


pub type JsefResult<T = ()> = Result<T, JsefErr>;


#[derive(Debug, Clone)]
pub struct JsefErr {
	pub err: JsefErrType,
	pub pos: usize,
}

impl JsefErr {
	pub const fn new(err: JsefErrType, pos: usize) -> Self {
		Self {err, pos}
	}
}

impl fmt::Display for JsefErr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "JSeF error at {}: {}", self.pos, self.err)
	}
}

impl error::Error for JsefErr {}


#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum JsefErrType {
	BadChar(char),
	BadEof,
	MaxDepth,
}

impl fmt::Display for JsefErrType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::BadChar(c) => write!(f, "unexpected character '{c}'"),
			Self::BadEof => write!(f, "unexpected EOF"),
			Self::MaxDepth => write!(f, "maximum nesting depth exceeded"),
		}
	}
}

impl error::Error for JsefErrType {}
