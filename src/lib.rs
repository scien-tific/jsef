#![doc = include_str!("../README.md")]


mod test;
mod err;
mod io;
mod counter;
mod value;
mod parse;
mod compose;

pub use err::*;
pub use value::*;
pub use compose::ComposeOpts;

use crash::CrashMap;
use parse::Parser;
use compose::Composer;


/// Maximum nesting level for parsing and composing.
#[cfg(not(test))]
pub const DEPTH_LIMIT: usize = 256;

#[cfg(test)]
pub const DEPTH_LIMIT: usize = 2;


/// A list of [`JsefValue`]s.
pub type JsefList = Vec<JsefValue>;

/// A string-keyed map of [`JsefValue`]s.
pub type JsefDict = CrashMap<String, JsefValue>;


/// Parses a [`JsefValue`] from the input string.
/// 
/// Requires root lists and dicts to be enclosed in the appropriate brackets.
pub fn parse_value<S>(string: &S) -> JsefResult<JsefValue>
where S: AsRef<str> + ?Sized {
	Parser::new(string.as_ref()).parse_value_root()
}

/// Parses a [`JsefList`] from the input string.
/// 
/// *Requires* the square brackets around the root list to be omitted.
pub fn parse_list<S>(string: &S) -> JsefResult<JsefList>
where S: AsRef<str> + ?Sized {
	Parser::new(string.as_ref()).parse_list_root()
}

/// Parses a [`JsefDict`] from the input string.
/// 
/// *Requires* the curly brackets around the root dict to be omitted.
pub fn parse_dict<S>(string: &S) -> JsefResult<JsefDict>
where S: AsRef<str> + ?Sized {
	Parser::new(string.as_ref()).parse_dict_root()
}


/// Composes the input [`JsefValue`] into a string formatted using [`opts`](ComposeOpts).
/// 
/// Includes root brackets and acts as a counterpart to [`parse_value`].
pub fn compose_value(value: &JsefValue, opts: &ComposeOpts) -> JsefResult<String> {
	Composer::new(opts).compose_value_root(value)
}

/// Composes the input [`JsefList`] into a string formatted using [`opts`](ComposeOpts).
/// 
/// Omits root square brackets and acts as a counterpart to [`parse_list`].
pub fn compose_list(list: &JsefList, opts: &ComposeOpts) -> JsefResult<String> {
	Composer::new(opts).compose_list_root(list)
}

/// Composes the input [`JsefDict`] into a string formatted using [`opts`](ComposeOpts).
/// 
/// Omits root curly brackets and acts as a counterpart to [`parse_dict`].
pub fn compose_dict(dict: &JsefDict, opts: &ComposeOpts) -> JsefResult<String> {
	Composer::new(opts).compose_dict_root(dict)
}


fn is_word_char(c: char) -> bool {
	const SPECIAL: [char; 8] = ['"', '=', '.', '{', '}', '[', ']', '#'];
	!c.is_ascii_whitespace() && !SPECIAL.contains(&c)
}
