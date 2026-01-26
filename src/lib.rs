#![doc = include_str!("../README.md")]


mod err;
mod parse;
mod compose;
mod test;

pub use err::*;
pub use compose::ComposeOpts;

use crash::CrashMap;
use parse::Parser;
use compose::Composer;


/// Maximum nesting level for parsing and composing.
pub const DEPTH_LIMIT: usize = 256;

// Might be useful to eventually change these to wrapper structs

/// A list of [`JsefValue`]s.
pub type JsefList = Vec<JsefValue>;

/// A string-keyed map of [`JsefValue`]s.
pub type JsefDict = CrashMap<String, JsefValue>;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsefValue {
	String(String),
	List(JsefList),
	Dict(JsefDict),
}

macro_rules! get {
	( $self:expr, $var:pat => $out:expr ) => {
		match $self {
			$var => Some($out),
			_ => None,
		}
	};
}

macro_rules! take {
	( $self:expr, $var:pat => $out:expr ) => {
		match $self {
			$var => Ok($out),
			other => Err(other),
		}
	};
}

impl JsefValue {
	pub fn new_string() -> Self {
		Self::String(String::new())
	}
	
	pub fn new_list() -> Self {
		Self::List(JsefList::new())
	}
	
	pub fn new_dict() -> Self {
		Self::Dict(JsefDict::default())
	}
	
	
	pub fn is_string(&self) -> bool {
		matches!(self, Self::String(_))
	}
	
	pub fn as_string(&self) -> Option<&String> {
		get!(self, Self::String(s) => s)
	}
	
	pub fn as_string_mut(&mut self) -> Option<&mut String> {
		get!(self, Self::String(s) => s)
	}
	
	pub fn take_string(self) -> Result<String, Self> {
		take!(self, Self::String(s) => s)
	}
	
	
	pub fn is_list(&self) -> bool {
		matches!(self, Self::List(_))
	}
	
	pub fn as_list(&self) -> Option<&JsefList> {
		get!(self, Self::List(l) => l)
	}
	
	pub fn as_list_mut(&mut self) -> Option<&mut JsefList> {
		get!(self, Self::List(l) => l)
	}
	
	pub fn take_list(self) -> Result<JsefList, Self> {
		take!(self, Self::List(l) => l)
	}
	
	
	pub fn is_dict(&self) -> bool {
		matches!(self, Self::Dict(_))
	}
	
	pub fn as_dict(&self) -> Option<&JsefDict> {
		get!(self, Self::Dict(d) => d)
	}
	
	pub fn as_dict_mut(&mut self) -> Option<&mut JsefDict> {
		get!(self, Self::Dict(d) => d)
	}
	
	pub fn take_dict(self) -> Result<JsefDict, Self> {
		take!(self, Self::Dict(d) => d)
	}
}

impl From<String> for JsefValue {
	fn from(string: String) -> Self {
		Self::String(string)
	}
}

impl From<&str> for JsefValue {
	fn from(string: &str) -> Self {
		Self::String(string.to_owned())
	}
}

impl From<JsefList> for JsefValue {
	fn from(list: JsefList) -> Self {
		Self::List(list)
	}
}

impl From<&[JsefValue]> for JsefValue {
	fn from(slice: &[JsefValue]) -> Self {
		Self::List(JsefList::from(slice))
	}
}

impl<const N: usize> From<[JsefValue; N]> for JsefValue {
	fn from(arr: [JsefValue; N]) -> Self {
		Self::List(JsefList::from(arr))
	}
}

impl From<JsefDict> for JsefValue {
	fn from(dict: JsefDict) -> Self {
		Self::Dict(dict)
	}
}


/// Parses any [`JsefValue`] from the input string.
/// 
/// Requires root lists and dicts to be enclosed in the appropriate brackets.
pub fn parse_value<S>(string: S) -> JsefResult<JsefValue>
where S: AsRef<str> {
	Parser::new(string.as_ref()).parse_value_root()
}

/// Parses a [`JsefList`] from the input string.
/// 
/// *Requires* root square brackets to be omitted.
pub fn parse_list<S>(string: S) -> JsefResult<JsefList>
where S: AsRef<str> {
	Parser::new(string.as_ref()).parse_list_root()
}

/// Parses a [`JsefDict`] from the input string.
/// 
/// *Requires* root curly brackets to be omitted.
pub fn parse_dict<S>(string: S) -> JsefResult<JsefDict>
where S: AsRef<str> {
	Parser::new(string.as_ref()).parse_dict_root()
}


/// Composes the input [`JsefValue`] into a string formatted using [`opts`](ComposeOpts).
/// 
/// Includes root brackets and pairs with [`parse_value`].
pub fn compose_value(value: &JsefValue, opts: &ComposeOpts) -> JsefResult<String> {
	Composer::new(opts).compose_value_root(value)
}

/// Composes the input [`JsefList`] into a string formatted using [`opts`](ComposeOpts).
/// 
/// Omits root brackets and pairs with [`parse_list`].
pub fn compose_list(list: &JsefList, opts: &ComposeOpts) -> JsefResult<String> {
	Composer::new(opts).compose_list_root(list)
}

/// Composes the input [`JsefDict`] into a string formatted using [`opts`](ComposeOpts).
/// 
/// Omits root brackets and pairs with [`parse_dict`].
pub fn compose_dict(dict: &JsefDict, opts: &ComposeOpts) -> JsefResult<String> {
	Composer::new(opts).compose_dict_root(dict)
}


fn is_word_char(c: char) -> bool {
	const SPECIAL: [char; 8] = ['"', '=', '.', '{', '}', '[', ']', '#'];
	!c.is_ascii_whitespace() && !SPECIAL.contains(&c)
}
