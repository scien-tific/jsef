use crate::{
	JsefValue, JsefList, JsefDict,
	JsefErr, JsefErrType, JsefResult,
	DEPTH_LIMIT, is_word_char,
};


/// Formatting options for composing [`JsefValue`]s into strings.
#[derive(Debug, Clone)]
pub struct ComposeOpts<'a> {
	/// Used for indenting newlines.
	/// `None` means the entire JSeF will be composed on a single line.
	pub indent: Option<&'a str>,
	
	/// Whether all keys and string values should be enclosed in double quotes regardless of their content.
	pub force_quotes: bool,
	
	/// Whether extra spaces should be omitted when unnecessary.
	pub dense: bool,
	
	/// Whether root elements should be enclosed in appropriate brackets.
	/// Has no effect on [`compose_value`](crate::compose_value).
	pub root_brackets: bool,
	
	/// Whether single-item dicts should be folded with the path notation.
	pub fold_dicts: bool,
}

impl ComposeOpts<'static> {
	/// The default options for readable, "pretty" outputs.
	/// 
	/// # Values
	/// - `indent`: `Some("\t")`
	/// - `force_quotes`: `false`
	/// - `dense`: `false`
	/// - `root_brackets`: `false`
	/// - `fold_dicts`: `true`
	pub const PRETTY: Self = Self {
		indent: Some("\t"),
		force_quotes: false,
		dense: false,
		root_brackets: false,
		fold_dicts: true,
	};
	
	/// The default options for compact outputs not necessarily intended for reading.
	/// 
	/// # Values
	/// - `indent`: `None`
	/// - `force_quotes`: `false`
	/// - `dense`: `true`
	/// - `root_brackets`: `false`
	/// - `fold_dicts`: `true`
	pub const COMPACT: Self = Self {
		indent: None,
		force_quotes: false,
		dense: true,
		root_brackets: false,
		fold_dicts: true,
	};
	
	/// The default options for simplified outputs that are easier to parse.
	/// 
	/// # Values
	/// - `indent`: `None`
	/// - `force_quotes`: `true`
	/// - `dense`: `true`
	/// - `root_brackets`: `true`
	/// - `fold_dicts`: `false`
	pub const SIMPLE: Self = Self {
		indent: None,
		force_quotes: true,
		dense: true,
		root_brackets: true,
		fold_dicts: false,
	};
}

impl<'a> ComposeOpts<'a> {
	pub fn indent<T>(mut self, value: T) -> Self
	where T: Into<Option<&'a str>> {
		self.indent = value.into();
		self
	}
	
	pub fn force_quotes(mut self, value: bool) -> Self {
		self.force_quotes = value;
		self
	}
	
	pub fn dense(mut self, value: bool) -> Self {
		self.dense = value;
		self
	}
	
	pub fn root_brackets(mut self, value: bool) -> Self {
		self.root_brackets = value;
		self
	}
	
	pub fn fold_dicts(mut self, value: bool) -> Self {
		self.fold_dicts = value;
		self
	}
}


#[derive(Debug)]
pub(crate) struct Composer<'o> {
	opts: &'o ComposeOpts<'o>,
	target: String,
	depth: usize,
}

impl<'o> Composer<'o> {
	pub(crate) fn new(opts: &'o ComposeOpts) -> Self {
		Self {target: String::new(), depth: 0, opts}
	}
	
	pub(crate) fn compose_value_root(mut self, value: &JsefValue) -> JsefResult<String> {
		self.compose_value(value)?;
		Ok(self.target)
	}
	
	pub(crate) fn compose_list_root(mut self, list: &JsefList) -> JsefResult<String> {
		let brackets = self.opts.root_brackets;
		self.compose_list(list, brackets)?;
		
		Ok(self.target)
	}
	
	pub(crate) fn compose_dict_root(mut self, dict: &JsefDict) -> JsefResult<String> {
		let brackets = self.opts.root_brackets;
		self.compose_dict(dict, brackets)?;
		
		Ok(self.target)
	}
}

impl Composer<'_> {
	fn err(&self, err: JsefErrType) -> JsefErr {
		JsefErr::new(err, self.target.len())
	}
	
	fn push_depth(&mut self) -> JsefResult {
		self.depth += 1;
		
		if self.depth < DEPTH_LIMIT {
			Ok(())
		} else {
			Err(self.err(JsefErrType::MaxDepth))
		}
	}
	
	fn pop_depth(&mut self) {
		self.depth -= 1;
	}
	
	fn separator(&mut self, space: bool) {
		if let Some(indent) = self.opts.indent {
			let len = indent.len() * self.depth;
			self.target.reserve(len + 1);
			self.target.push('\n');
			
			for _ in 0..self.depth {
				self.target.push_str(indent);
			}
		} else if space || !self.opts.dense {
			self.target.push(' ');
		}
	}
	
	fn escape_string(&mut self, string: &str) {
		self.target.reserve(string.len());
		
		for c in string.chars() {
			match c {
				'\n' => self.target.push_str("\\n"),
				'\t' => self.target.push_str("\\t"),
				'\r' => self.target.push_str("\\r"),
				'\0' => self.target.push_str("\\0"),
				'\\' => self.target.push_str("\\\\"),
				'"' => self.target.push_str("\\\""),
				
				c => self.target.push(c),
			}
		}
	}
	
	fn compose_string(&mut self, string: &str) {
		let quotes = self.opts.force_quotes ||
			string.chars().any(|c| !is_word_char(c));
		
		if quotes {
			let len = string.len() + 2;
			self.target.reserve(len);
			self.target.push('"');
			self.escape_string(string);
			self.target.push('"');
		} else {
			self.escape_string(string);
		}
	}
	
	fn compose_pair(&mut self, key: &str, mut value: &JsefValue) -> JsefResult {
		self.compose_string(key);
		
		if self.opts.fold_dicts {
			while let Some(dict) = value.as_dict() {
				if dict.len() != 1 {break;}
				
				// dict.len() == 1 here, so unwrap should be ok
				let (key, val) = dict.iter().next().unwrap();
				self.target.push('.');
				self.target.push_str(key);
				value = val;
			}
		}
		
		if self.opts.dense {
			self.target.push('=');
		} else {
			self.target.push_str(" = ");
		}
		
		self.compose_value(value)?;
		Ok(())
	}
	
	fn compose_many<I, F>(
		&mut self,
		brackets: bool,
		open: char, close: char,
		mut iter: I, mut func: F
	) -> JsefResult
	where
		I: Iterator,
		F: FnMut(&mut Self, I::Item) -> JsefResult,
	{
		let mut empty = true;
		self.push_depth()?;
		
		if brackets {
			self.target.push(open);
		}
		
		if let Some(it) = iter.next() {
			empty = false;
			self.separator(false);
			func(self, it)?;
		}
		
		for it in iter {
			self.separator(true);
			func(self, it)?;
		}
		
		if brackets {
			if !empty {self.separator(false);}
			self.target.push(close);
		}
		
		self.pop_depth();
		
		Ok(())
	}
	
	fn compose_value(&mut self, value: &JsefValue) -> JsefResult {
		match value {
			JsefValue::String(string) => Ok(self.compose_string(string)),
			JsefValue::List(list) => self.compose_list(list, true),
			JsefValue::Dict(dict) => self.compose_dict(dict, true),
		}
	}
	
	fn compose_list(&mut self, list: &JsefList, brackets: bool) -> JsefResult {
		self.compose_many(brackets, '[', ']', list.iter(),
			|this, val| this.compose_value(val)
		)
	}
	
	fn compose_dict(&mut self, dict: &JsefDict, brackets: bool) -> JsefResult {
		self.compose_many(brackets, '{', '}', dict.iter(),
			|this, (key, val)| this.compose_pair(key, val)
		)
	}
}
