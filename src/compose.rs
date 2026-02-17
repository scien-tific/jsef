use crate::{
	JsefValue, JsefList, JsefDict,
	JsefErrType::{self, *},
	JsefErr, JsefResult,
	DEPTH_LIMIT,
	is_word_char, count_line_col,
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
	pub const fn indent(mut self, value: &'a str) -> Self {
		self.indent = Some(value);
		self
	}
	
	pub const fn no_indent(mut self) -> Self {
		self.indent = None;
		self
	}
	
	pub const fn force_quotes(mut self, value: bool) -> Self {
		self.force_quotes = value;
		self
	}
	
	pub const fn dense(mut self, value: bool) -> Self {
		self.dense = value;
		self
	}
	
	pub const fn fold_dicts(mut self, value: bool) -> Self {
		self.fold_dicts = value;
		self
	}
	
	pub const fn prelude(mut self, value: &'a str) -> Self {
		self.prelude = Some(value);
		self
	}
	
	pub const fn no_prelude(mut self) -> Self {
		self.prelude = None;
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
	fn err(&self, err: JsefErrType) -> JsefErr {
		let (line, col) = count_line_col(&self.target);
		JsefErr::new(err, line, col)
	}
	
	fn separator(&mut self, space: bool) {
		if let Some(indent) = self.opts.indent {
			let len = indent.len() * self.depth + 1;
			self.target.reserve(len);
			self.target.push_str("\n");
			
			for _ in 0..self.depth {
				self.target.push_str(indent);
			}
		} else if space || !self.opts.dense {
			self.target.push_str(" ");
		}
	}
	
	fn compose_prelude(&mut self) {
		if let Some(msg) = self.opts.prelude {
			for line in msg.lines() {
				self.target.push_str("# ");
				self.target.push_str(line);
				self.target.push_str("\n");
			}
		}
	}
	
	fn escape_string(&mut self, string: &str) {
		let mut idx = 0;
		for (i, c) in string.char_indices() {
			let esc = match c {
				'\n' => "\\n",
				'\t' => "\\t",
				'\r' => "\\r",
				'\0' => "\\0",
				'\\' => "\\\\",
				'"' => "\\\"",
				
				_ => continue,
			};
			
			let slice = &string[idx..i];
			self.target.push_str(slice);
			self.target.push_str(esc);
			
			// CHANGE THIS if escaped chars can be more than one byte long
			idx = i + 1;
		}
		
		let slice = &string[idx..];
		self.target.push_str(slice);
	}
	
	fn compose_string(&mut self, string: &str) {
		let quotes = self.opts.force_quotes ||
			string.chars().any(|c| !is_word_char(c));
		
		if quotes {
			self.target.push_str("\"");
			self.escape_string(string);
			self.target.push_str("\"");
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
				self.target.push_str(".");
				self.target.push_str(key);
				value = val;
			}
		}
		
		if self.opts.dense {
			self.target.push_str("=");
		} else {
			self.target.push_str(" = ");
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
			self.depth += 1;
			if self.depth > DEPTH_LIMIT {
				return Err(self.err(MaxDepth));
			}
			
			self.target.push(open);
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
			self.depth -= 1;
			if !empty {self.separator(false);}
			self.target.push(close);
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
