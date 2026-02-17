use crate::{
	JsefValue, JsefList, JsefDict,
	JsefErrType::{self, *},
	JsefErr, JsefResult,
	DEPTH_LIMIT,
	is_word_char, count_line_col,
};


#[derive(Debug)]
pub(crate) struct Parser<'s> {
	source: &'s str,
	peek: Option<char>,
	idx: usize,
	depth: usize,
}

impl<'s> Parser<'s> {
	pub(crate) fn new(source: &'s str) -> Self {
		let peek = source.chars().next();
		
		Self {
			idx: 0, depth: 0,
			source, peek,
		}
	}
	
	pub(crate) fn parse_value_root(mut self) -> JsefResult<JsefValue> {
		self.skip_whitespace();
		let value = self.parse_value()?;
		self.skip_whitespace();
		self.assert_eof()?;
		
		Ok(value)
	}
	
	pub(crate) fn parse_list_root(mut self) -> JsefResult<JsefList> {
		let list = self.parse_list(true)?;
		self.skip_whitespace();
		self.assert_eof()?;
		
		Ok(list)
	}
	
	pub(crate) fn parse_dict_root(mut self) -> JsefResult<JsefDict> {
		let dict = self.parse_dict(true)?;
		self.skip_whitespace();
		self.assert_eof()?;
		
		Ok(dict)
	}
}

impl Parser<'_> {
	fn err(&self, err: JsefErrType) -> JsefErr {
		let (line, col) = count_line_col(&self.source[..self.idx]);
		JsefErr::new(err, line, col)
	}
	
	fn slice(&self) -> &str {
		&self.source[self.idx..]
	}
	
	fn peek(&self) -> Option<char> {
		self.peek
	}
	
	fn next(&mut self) -> Option<char> {
		let prev = self.peek;
		self.idx = self.source.ceil_char_boundary(self.idx + 1);
		self.peek = self.slice().chars().next();
		
		prev
	}
	
	fn take(&mut self) -> JsefResult<char> {
		self.next().ok_or_else(|| self.err(Unexpected(None)))
	}
	
	fn next_while<F>(&mut self, mut pred: F) -> &str
	where F: FnMut(char) -> bool {
		let slice = self.slice();
		let len = slice.len();
		let start = self.idx;
		
		let find = slice
			.char_indices()
			.find(|(_, c)| !pred(*c));
		
		if let Some((i, c)) = find {
			self.peek = Some(c);
			self.idx += i;
		} else {
			self.peek = None;
			self.idx += len;
		}
		
		&self.source[start..self.idx]
	}
	
	fn eat(&mut self, c: char) -> JsefResult {
		// Can't call Self::next right away,
		// since that would screw up the error line-column reporting
		match self.peek() {
			Some(p) if p == c => {
				self.next();
				Ok(())
			},
			
			p => Err(self.err(Mismatch(c, p))),
		}
	}
	
	fn try_eat(&mut self, c: char) -> bool {
		if self.peek() == Some(c) {
			self.next();
			true
		} else {
			false
		}
	}
	
	fn assert_eof(&self) -> JsefResult {
		match self.peek() {
			Some(p) => Err(self.err(NotEof(p))),
			None => Ok(()),
		}
	}
	
	fn skip_whitespace(&mut self) {
		while let Some(c) = self.peek() {
			if c.is_ascii_whitespace() {
				self.next_while(|c| c.is_ascii_whitespace());
				continue;
			}
			
			if c == '#' {
				self.next_while(|c| c != '\n');
				continue;
			}
			
			break;
		}
	}
	
	fn parse_word(&mut self) -> JsefResult<String> {
		let slice = self.next_while(is_word_char);
		
		if !slice.is_empty() {
			Ok(slice.to_owned())
		} else {
			Err(self.err(Unexpected(self.peek())))
		}
	}
	
	fn parse_escape(&mut self) -> JsefResult<char> {
		self.eat('\\')?;
		
		match self.take()? {
			'n' => Ok('\n'),
			't' => Ok('\t'),
			'r' => Ok('\r'),
			'0' => Ok('\0'),
			
			c => Ok(c),
		}
	}
	
	fn parse_string(&mut self) -> JsefResult<String> {
		let mut string = String::new();
		self.eat('"')?;
		
		loop {
			let slice = self.next_while(|c| c != '"' && c != '\\');
			string.push_str(slice);
			
			if self.peek() == Some('\\') {
				let c = self.parse_escape()?;
				string.push(c);
			} else {
				self.eat('"')?;
				break;
			}
		}
		
		Ok(string)
	}
	
	fn parse_ident(&mut self) -> JsefResult<String> {
		match self.peek() {
			Some('"') => self.parse_string(),
			_ => self.parse_word(),
		}
	}
	
	fn parse_pair(&mut self, mut dict: &mut JsefDict) -> JsefResult {
		let mut key = self.parse_ident()?;
		self.skip_whitespace();
		
		while self.try_eat('.') {
			let value = dict
				.entry(key)
				.or_insert_with(JsefValue::new_dict);
			
			match value {
				JsefValue::Dict(d) => dict = d,
				
				val => {
					*val = JsefValue::new_dict();
					// unwrap should be safe, val was just replaced with a JsefValue::Dict
					dict = val.as_dict_mut().unwrap();
				},
			}
			
			self.skip_whitespace();
			key = self.parse_ident()?;
			self.skip_whitespace();
		}
		
		self.eat('=')?;
		self.skip_whitespace();
		
		let value = self.parse_value()?;
		dict.insert(key, value);
		
		Ok(())
	}
	
	fn parse_many<P, F>(
		&mut self,
		root: bool, open: char, close: char,
		mut pred: P, mut func: F,
	) -> JsefResult
	where
		P: FnMut(char) -> bool,
		F: FnMut(&mut Self) -> JsefResult,
	{
		if !root {
			self.depth += 1;
			if self.depth > DEPTH_LIMIT {
				return Err(self.err(MaxDepth));
			}
			
			self.eat(open)?;
		}
		
		self.skip_whitespace();
		
		while self.peek().is_some_and(&mut pred) {
			func(self)?;
			self.skip_whitespace();
		}
		
		if !root {
			self.depth -= 1;
			self.eat(close)?;
		}
		
		Ok(())
	}
	
	fn parse_value(&mut self) -> JsefResult<JsefValue> {
		match self.peek() {
			Some('{') => Ok(JsefValue::Dict(self.parse_dict(false)?)),
			Some('[') => Ok(JsefValue::List(self.parse_list(false)?)),
			Some('"') => Ok(JsefValue::String(self.parse_string()?)),
			Some(_) => Ok(JsefValue::String(self.parse_word()?)),
			
			p => Err(self.err(Unexpected(p))),
		}
	}
	
	fn parse_list(&mut self, root: bool) -> JsefResult<JsefList> {
		let mut list = JsefList::new();
		self.parse_many(root, '[', ']',
			|c| c == '"' || c == '[' || c == '{' || is_word_char(c),
			|this| {
				let value = this.parse_value()?;
				list.push(value);
				Ok(())
			},
		)?;
		
		Ok(list)
	}
	
	fn parse_dict(&mut self, root: bool) -> JsefResult<JsefDict> {
		let mut dict = JsefDict::default();
		self.parse_many(root, '{', '}',
			|c| c == '"' || is_word_char(c),
			|this| this.parse_pair(&mut dict),
		)?;
		
		Ok(dict)
	}
}
