use crate::{
	JsefValue, JsefList, JsefDict,
	JsefErrType, JsefResult,
	DEPTH_LIMIT, is_word_char,
	counter::LineColCounter,
};


/// Formatting options for composing [`JsefValue`]s into strings.
#[derive(Debug, Clone)]
pub struct ComposeOpts<'a> {
	/// Used for indenting newlines.
	/// `None` means the entire JSeF will be composed on a single line.
	pub indent: Option<&'a str>,
	
	/// Whether all keys and values should be enclosed in double quotes regardless of their content.
	pub force_quotes: bool,
	
	/// Whether extra spaces should be omitted when unnecessary.
	pub dense: bool,
	
	/// Whether single-item dicts should be folded with the path notation.
	pub fold_dicts: bool,
	
	/// A message that is written at the start of the composed string using line comments.
	pub prelude: Option<&'a str>,
}

impl ComposeOpts<'static> {
	/// The default options for readable, "pretty" outputs.
	/// 
	/// # Values
	/// - `indent`: `Some("\t")`
	/// - `force_quotes`: `false`
	/// - `dense`: `false`
	/// - `fold_dicts`: `true`
	/// - `prelude`: `None`
	pub const PRETTY: Self = Self {
		indent: Some("\t"),
		force_quotes: false,
		dense: false,
		fold_dicts: true,
		prelude: None,
	};
	
	/// The default options for compact outputs not necessarily intended for reading.
	/// 
	/// # Values
	/// - `indent`: `None`
	/// - `force_quotes`: `false`
	/// - `dense`: `true`
	/// - `fold_dicts`: `true`
	/// - `prelude`: `None`
	pub const COMPACT: Self = Self {
		indent: None,
		force_quotes: false,
		dense: true,
		fold_dicts: true,
		prelude: None,
	};
	
	/// The default options for simplified outputs that are easier to parse.
	/// 
	/// # Values
	/// - `indent`: `None`
	/// - `force_quotes`: `true`
	/// - `dense`: `true`
	/// - `fold_dicts`: `false`
	/// - `prelude`: `None`
	pub const SIMPLE: Self = Self {
		indent: None,
		force_quotes: true,
		dense: true,
		fold_dicts: false,
		prelude: None,
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
	
	pub fn fold_dicts(mut self, value: bool) -> Self {
		self.fold_dicts = value;
		self
	}
	
	pub fn prelude<T>(mut self, value: T) -> Self
	where T: Into<Option<&'a str>> {
		self.prelude = value.into();
		self
	}
}


#[derive(Debug)]
pub(crate) struct Composer<'o> {
	opts: &'o ComposeOpts<'o>,
	target: String,
	depth: usize,
	counter: LineColCounter,
}

impl<'o> Composer<'o> {
	pub(crate) fn new(opts: &'o ComposeOpts) -> Self {
		Self {
			target: String::new(),
			depth: 0,
			counter: LineColCounter::new(),
			opts,
		}
	}
	
	pub(crate) fn compose_value_root(mut self, value: &JsefValue) -> JsefResult<String> {
		self.compose_prelude();
		self.compose_value(value)?;
		Ok(self.target)
	}
	
	pub(crate) fn compose_list_root(mut self, list: &JsefList) -> JsefResult<String> {
		self.compose_prelude();
		self.compose_list(list, true)?;
		
		Ok(self.target)
	}
	
	pub(crate) fn compose_dict_root(mut self, dict: &JsefDict) -> JsefResult<String> {
		self.compose_prelude();
		self.compose_dict(dict, true)?;
		
		Ok(self.target)
	}
}

impl Composer<'_> {
	fn push_depth(&mut self) -> JsefResult {
		self.depth += 1;
		
		if self.depth < DEPTH_LIMIT {
			Ok(())
		} else {
			Err(self.counter.err(JsefErrType::MaxDepth))
		}
	}
	
	fn pop_depth(&mut self) {
		self.depth -= 1;
	}
	
	fn write_char(&mut self, c: char) {
		self.counter.count(c);
		self.target.push(c);
	}
	
	fn write(&mut self, slice: &str) {
		self.counter.count_str(slice);
		self.target.push_str(slice);
	}
	
	fn separator(&mut self, space: bool) {
		if let Some(indent) = self.opts.indent {
			let len = indent.len() * self.depth + 1;
			self.target.reserve(len);
			self.write_char('\n');
			
			for _ in 0..self.depth {
				self.write(indent);
			}
		} else if space || !self.opts.dense {
			self.write_char(' ');
		}
	}
	
	fn compose_prelude(&mut self) {
		if let Some(msg) = self.opts.prelude {
			for line in msg.lines() {
				let len = line.len() + 3;
				self.target.reserve(len);
				self.write("# ");
				self.write(line);
				self.write_char('\n');
			}
		}
	}
	
	fn escape_string(&mut self, string: &str) {
		self.target.reserve(string.len());
		
		for c in string.chars() {
			match c {
				'\n' => self.write("\\n"),
				'\t' => self.write("\\t"),
				'\r' => self.write("\\r"),
				'\0' => self.write("\\0"),
				'\\' => self.write("\\\\"),
				'"' => self.write("\\\""),
				
				c => self.write_char(c),
			}
		}
	}
	
	fn compose_string(&mut self, string: &str) {
		let quotes = self.opts.force_quotes ||
			string.chars().any(|c| !is_word_char(c));
		
		if quotes {
			let len = string.len() + 2;
			self.target.reserve(len);
			self.write_char('"');
			self.escape_string(string);
			self.write_char('"');
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
				self.write_char('.');
				self.write(key);
				value = val;
			}
		}
		
		if self.opts.dense {
			self.write_char('=');
		} else {
			self.write(" = ");
		}
		
		self.compose_value(value)?;
		Ok(())
	}
	
	fn compose_many<I, F>(
		&mut self,
		root: bool,
		open: char, close: char,
		mut iter: I, mut func: F
	) -> JsefResult
	where
		I: Iterator,
		F: FnMut(&mut Self, I::Item) -> JsefResult,
	{
		let mut empty = true;
		
		if !root {
			self.push_depth()?;
			self.write_char(open);
		}
		
		if let Some(it) = iter.next() {
			empty = false;
			if !root {self.separator(false);}
			func(self, it)?;
		}
		
		for it in iter {
			self.separator(true);
			func(self, it)?;
		}
		
		if !root {
			self.pop_depth();
			if !empty {self.separator(false);}
			self.write_char(close);
		}
		
		Ok(())
	}
	
	fn compose_value(&mut self, value: &JsefValue) -> JsefResult {
		match value {
			JsefValue::String(string) => Ok(self.compose_string(string)),
			JsefValue::List(list) => self.compose_list(list, false),
			JsefValue::Dict(dict) => self.compose_dict(dict, false),
		}
	}
	
	fn compose_list(&mut self, list: &JsefList, root: bool) -> JsefResult {
		self.compose_many(root, '[', ']', list.iter(),
			|this, val| this.compose_value(val)
		)
	}
	
	fn compose_dict(&mut self, dict: &JsefDict, root: bool) -> JsefResult {
		self.compose_many(root, '{', '}', dict.iter(),
			|this, (key, val)| this.compose_pair(key, val)
		)
	}
}
