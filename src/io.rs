use std::io::{self, Read, Bytes};


// https://en.wikipedia.org/wiki/UTF-8
#[derive(Debug)]
pub(crate) struct CharReader<R> where R: Read {
	bytes: Bytes<R>,
}

impl<R: Read> CharReader<R> {
	pub(crate) fn new(reader: R) -> Self {
		Self {bytes: reader.bytes()}
	}
}

impl<R: Read> CharReader<R> {
	fn read_cont(&mut self) -> io::Result<u32> {
		use io::ErrorKind::InvalidData;
		
		let byte = self.bytes.next()
			.ok_or_else(|| io::Error::from(InvalidData))??;
		
		match byte {
			0x80..=0xBF => Ok(u32::from(byte & 0x3F)),
			_ => Err(InvalidData.into()),
		}
	}
	
	fn read_code(&mut self) -> io::Result<Option<char>> {
		use io::ErrorKind::InvalidData;
		
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
			_ => return Err(InvalidData.into()),
		};
		
		char::from_u32(code)
			.ok_or_else(|| io::Error::from(InvalidData))
			.map(|c| Some(c))
	}
}

impl<R: Read> Iterator for CharReader<R> {
	type Item = io::Result<char>;
	
	fn next(&mut self) -> Option<Self::Item> {
		// Implementation works with a transposed return type for easier error propagation
		self.read_code().transpose()
	}
}
