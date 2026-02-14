use crate::JsefErrType::{self, InvalidUtf8};
use std::io::{Read, Bytes};


// https://en.wikipedia.org/wiki/UTF-8
#[derive(Debug)]
pub(crate) struct CharReader<R> where R: Read {
	bytes: Bytes<R>,
}

impl<R: Read> CharReader<R> {
	pub(crate) fn new(reader: R) -> Self {
		Self {bytes: reader.bytes()}
	}
	
	pub(crate) fn read_char(&mut self) -> Result<Option<char>, JsefErrType> {
		let byte = self.bytes.next().transpose()?;
		let code = match byte {
			Some(b @ 0x00..=0x7F) => return Ok(Some(char::from(b))),
			
			Some(b @ 0xC0..=0xDF) =>
				(u32::from(b & 0x1F) << 6) |
				self.read_cont()?,
			
			Some(b @ 0xE0..=0xEF) =>
				(u32::from(b & 0x0F) << 12) |
				(self.read_cont()? << 6) |
				self.read_cont()?,
			
			Some(b @ 0xF0..=0xF7) =>
				(u32::from(b & 0x07) << 18) |
				(self.read_cont()? << 12) |
				(self.read_cont()? << 6) |
				self.read_cont()?,
			
			None => return Ok(None),
			_ => return Err(InvalidUtf8),
		};
		
		char::from_u32(code)
			.ok_or_else(|| InvalidUtf8)
			.map(|c| Some(c))
	}
}

impl<R: Read> CharReader<R> {
	fn read_cont(&mut self) -> Result<u32, JsefErrType> {
		let byte = self.bytes.next().transpose()?;
		
		match byte {
			Some(b @ 0x80..=0xBF) => Ok(u32::from(b & 0x3F)),
			_ => Err(InvalidUtf8),
		}
	}
}
