use crate::{
	JsefValue, JsefList, JsefDict,
	JsefErr, JsefErrType, JsefResult,
	DEPTH_LIMIT, is_word_char,
};


// Basic string slice based recursive descent parser
// Eventually should maybe be replaced with something based on io::(Buf)Read
#[derive(Debug)]
pub(crate) struct Parser<'s> {
	source: &'s str,
	idx: usize,
	peek: Option<char>,
	depth: usize,
}

impl<'s> Parser<'s> {
	pub(crate) fn new(source: &'s str) -> Self {
		let peek = source.chars().next();
		Self {idx: 0, depth: 0, source, peek}
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
		JsefErr::new(err, self.idx)
	}
	
	fn err_bad_char(&self) -> JsefErr {
		self.err(match self.peek() {
			Some(c) => JsefErrType::BadChar(c),
			None => JsefErrType::BadEof,
		})
	}
	
	fn at_eof(&self) -> bool {
		self.idx >= self.source.len()
	}
	
	fn peek(&self) -> Option<char> {
		// uhh
		self.peek
	}
	
	fn advance(&mut self) {
		self.idx = self.source.ceil_char_boundary(self.idx + 1);
		self.peek = self.source[self.idx..].chars().next();
	}
	
	fn move_to(&mut self, idx: usize) {
		self.idx = idx;
		self.peek = self.source[self.idx..].chars().next();
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
	
	fn take(&mut self) -> JsefResult<char> {
		match self.peek() {
			Some(c) => {
				self.advance();
				Ok(c)
			},
			
			None => Err(self.err(JsefErrType::BadEof)),
		}
	}
	
	fn take_while<F>(&mut self, mut pred: F) -> &str
	where F: FnMut(char) -> bool {
		let start = self.idx;
		let end = self.source[start..]
			.char_indices()
			.find(|(_, c)| !pred(*c))
			.map(|(i, _)| i + start)
			.unwrap_or(self.source.len());
		
		self.move_to(end);
		
		&self.source[start..end]
	}
	
	fn eat(&mut self, c: char) -> JsefResult {
		if self.peek() == Some(c) {
			self.advance();
			Ok(())
		} else {
			Err(self.err_bad_char())
		}
	}
	
	fn try_eat(&mut self, c: char) -> bool {
		if self.peek() == Some(c) {
			self.advance();
			true
		} else {
			false
		}
	}
	
	fn assert_eof(&self) -> JsefResult {
		match self.peek() {
			Some(c) => Err(self.err(JsefErrType::BadChar(c))),
			None => Ok(()),
		}
	}
	
	fn skip_whitespace(&mut self) {
		while let Some(c) = self.peek() {
			if c.is_ascii_whitespace() {
				self.take_while(|c| c.is_ascii_whitespace());
				continue;
			}
			
			if c == '#' {
				self.take_while(|c| c != '\n');
				continue;
			}
			
			break;
		}
	}
	
	fn parse_word(&mut self) -> JsefResult<String> {
		let slice = self.take_while(is_word_char);
		
		if !slice.is_empty() {
			Ok(slice.to_owned())
		} else {
			Err(self.err_bad_char())
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
			let part = self.take_while(|c| c != '"' && c != '\\');
			string.push_str(part);
			
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
		if self.peek() == Some('"') {
			self.parse_string()
		} else {
			self.parse_word()
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
	
	fn parse_many<F>(&mut self, root: bool, open: char, close: char, mut func: F) -> JsefResult
	where F: FnMut(&mut Self) -> JsefResult {
		if !root {
			self.push_depth()?;
			self.eat(open)?;
		}
		
		self.skip_whitespace();
		
		while if root {!self.at_eof()} else {!self.try_eat(close)} {
			func(self)?;
			self.skip_whitespace();
		}
		
		if !root {self.pop_depth();}
		
		Ok(())
	}
	
	fn parse_value(&mut self) -> JsefResult<JsefValue> {
		match self.peek() {
			Some('{') => Ok(JsefValue::Dict(self.parse_dict(false)?)),
			Some('[') => Ok(JsefValue::List(self.parse_list(false)?)),
			Some('"') => Ok(JsefValue::String(self.parse_string()?)),
			Some(_) => Ok(JsefValue::String(self.parse_word()?)),
			
			None => Err(self.err(JsefErrType::BadEof)),
		}
	}
	
	fn parse_list(&mut self, root: bool) -> JsefResult<JsefList> {
		let mut list = JsefList::new();
		self.parse_many(root, '[', ']', |this| {
			let value = this.parse_value()?;
			list.push(value);
			Ok(())
		})?;
		
		Ok(list)
	}
	
	fn parse_dict(&mut self, root: bool) -> JsefResult<JsefDict> {
		let mut dict = JsefDict::default();
		self.parse_many(root, '{', '}',
			|this| this.parse_pair(&mut dict)
		)?;
		
		Ok(dict)
	}
}
