use crate::{JsefErr, JsefErrType};


#[derive(Debug, Clone, Copy)]
pub(crate) struct LineColCounter {
	line: usize,
	col: usize,
}

impl LineColCounter {
	pub(crate) const fn new() -> Self {
		Self {line: 1, col: 1}
	}
	
	pub(crate) const fn err(&self, err: JsefErrType) -> JsefErr {
		JsefErr::new(err, self.line, self.col)
	}
	
	pub(crate) fn count(&mut self, c: char) {
		if c == '\n' {
			self.line += 1;
			self.col = 1;
		} else {
			self.col += 1;
		}
	}
	
	pub(crate) fn count_str(&mut self, slice: &str) {
		for c in slice.chars() {
			self.count(c);
		}
	}
}
