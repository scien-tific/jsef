use crate::{
	JsefValue, JsefList, JsefDict,
	JsefErrType, JsefResult,
	DEPTH_LIMIT, is_word_char,
	counter::LineColCounter,
	io::CharReader,
};
use std::io::Read;


#[derive(Debug)]
pub(crate) struct Parser<R> where R: Read {
	source: CharReader<R>,
	peek: Option<char>,
	counter: LineColCounter,
	depth: usize,
}

impl<R: Read> Parser<R> {
	pub(crate) fn new(source: R) -> JsefResult<Self> {
		let mut parser = Self {
			source: CharReader::new(source),
			peek: None,
			depth: 0,
			counter: LineColCounter::new(),
		};
		
		parser.advance()?;
		Ok(parser)
	}
	
	pub(crate) fn parse_value_root(mut self) -> JsefResult<JsefValue> {
		self.skip_whitespace()?;
		let value = self.parse_value()?;
		self.skip_whitespace()?;
		self.assert_eof()?;
		
		Ok(value)
	}
	
	pub(crate) fn parse_list_root(mut self) -> JsefResult<JsefList> {
		let list = self.parse_list(true)?;
		self.skip_whitespace()?;
		self.assert_eof()?;
		
		Ok(list)
	}
	
	pub(crate) fn parse_dict_root(mut self) -> JsefResult<JsefDict> {
		let dict = self.parse_dict(true)?;
		self.skip_whitespace()?;
		self.assert_eof()?;
		
		Ok(dict)
	}
}

impl<R: Read> Parser<R> {
	fn peek(&self) -> Option<char> {
		// uhh
		self.peek
	}
	
	fn advance(&mut self) -> JsefResult {
		if let Some(p) = self.peek() {
			self.counter.count(p);
		}
		
		self.peek = self.source.read_char()
			.map_err(|err| self.counter.err(err))?;
		
		Ok(())
	}
	
	fn push_depth(&mut self) -> JsefResult {
		self.depth += 1;
		
		if self.depth <= DEPTH_LIMIT {
			Ok(())
		} else {
			Err(self.counter.err(JsefErrType::MaxDepth))
		}
	}
	
	fn pop_depth(&mut self) {
		self.depth -= 1;
	}
	
	fn take(&mut self) -> JsefResult<char> {
		match self.peek() {
			Some(c) => {
				self.advance()?;
				Ok(c)
			},
			
			p => Err(self.counter.err(
				JsefErrType::Unexpected(p)
			)),
		}
	}
	
	fn skip_while<F>(&mut self, mut pred: F) -> JsefResult
	where F: FnMut(char) -> bool {
		while let Some(c) = self.peek() {
			if !pred(c) {break;}
			self.advance()?;
		}
		
		Ok(())
	}
	
	fn take_while<F>(&mut self, mut pred: F, string: &mut String) -> JsefResult
	where F: FnMut(char) -> bool {
		while let Some(c) = self.peek() {
			if !pred(c) {break;}
			string.push(c);
			self.advance()?;
		}
		
		Ok(())
	}
	
	fn eat(&mut self, c: char) -> JsefResult {
		let peek = self.peek();
		
		if peek == Some(c) {
			self.advance()?;
			Ok(())
		} else {
			Err(self.counter.err(
				JsefErrType::Mismatch(c, peek)
			))
		}
	}
	
	fn try_eat(&mut self, c: char) -> JsefResult<bool> {
		if self.peek() == Some(c) {
			self.advance()?;
			Ok(true)
		} else {
			Ok(false)
		}
	}
	
	fn assert_eof(&self) -> JsefResult {
		if let Some(c) = self.peek() {
			Err(self.counter.err(
				JsefErrType::NotEof(c)
			))
		} else {
			Ok(())
		}
	}
	
	fn skip_whitespace(&mut self) -> JsefResult {
		while let Some(c) = self.peek() {
			if c.is_ascii_whitespace() {
				self.advance()?;
				self.skip_while(|c| c.is_ascii_whitespace())?;
				continue;
			}
			
			if c == '#' {
				self.advance()?;
				self.skip_while(|c| c != '\n')?;
				continue;
			}
			
			break;
		}
		
		Ok(())
	}
	
	fn parse_word(&mut self) -> JsefResult<String> {
		let mut string = String::new();
		self.take_while(is_word_char, &mut string)?;
		
		if !string.is_empty() {
			Ok(string)
		} else {
			Err(self.counter.err(
				JsefErrType::Unexpected(self.peek())
			))
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
			self.take_while(
				|c| c != '"' && c != '\\',
				&mut string,
			)?;
			
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
		self.skip_whitespace()?;
		
		while self.try_eat('.')? {
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
			
			self.skip_whitespace()?;
			key = self.parse_ident()?;
			self.skip_whitespace()?;
		}
		
		self.eat('=')?;
		self.skip_whitespace()?;
		
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
			self.push_depth()?;
			self.eat(open)?;
		}
		
		self.skip_whitespace()?;
		
		while self.peek().is_some_and(&mut pred) {
			func(self)?;
			self.skip_whitespace()?;
		}
		
		if !root {
			self.pop_depth();
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
			
			p => Err(self.counter.err(
				JsefErrType::Unexpected(p)
			)),
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
