use crate::{JsefErr, JsefErrType};


/// This is just a zero-sized type when the `track-line-col` feature is disabled.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LineColCounter {
	#[cfg(not(feature = "no-line-col"))]
	line: usize,
	
	#[cfg(not(feature = "no-line-col"))]
	col: usize,
}

impl LineColCounter {
	#[cfg(not(feature = "no-line-col"))]
	pub(crate) const fn new() -> Self {
		Self {line: 1, col: 1}
	}
	
	#[cfg(feature = "no-line-col")]
	pub(crate) const fn new() -> Self {
		Self {}
	}
	
	#[cfg(not(feature = "no-line-col"))]
	pub(crate) const fn err(&self, err: JsefErrType) -> JsefErr {
		JsefErr::new(err, self.line, self.col)
	}
	
	#[cfg(feature = "no-line-col")]
	pub(crate) const fn err(&self, err: JsefErrType) -> JsefErr {
		JsefErr::new(err)
	}
	
	#[cfg(not(feature = "no-line-col"))]
	pub(crate) fn count(&mut self, c: char) {
		if c == '\n' {
			self.line += 1;
			self.col = 1;
		} else {
			self.col += 1;
		}
	}
	
	#[cfg(feature = "no-line-col")]
	pub(crate) fn count(&mut self, _c: char) {}
	
	#[cfg(not(feature = "no-line-col"))]
	pub(crate) fn count_str(&mut self, slice: &str) {
		for c in slice.chars() {
			self.count(c);
		}
	}
	
	#[cfg(feature = "no-line-col")]
	pub(crate) fn count_str(&mut self, _slice: &str) {}
}
