use std::io::{self, Read, Bytes};


#[derive(Debug)]
pub(crate) CharReader<R> where R: Read {
	bytes: Bytes<R>,
}

impl<R: Read> CharReader<R> {
	pub(crate) const fn new(reader: R) -> Self {
		Self {bytes: reader.bytes()}
	}
}

impl<R: Read> Iterator for CharReader<R> {
	type Item = io::Result<char>;
	
	fn next(&mut self) -> Option<Self::Item> {
		use io::ErrorKind::*;
		let byte = self.bytes.next()?;
		
		match byte {
			0b00000000..=0b01111111 => {
				let c = char::from(byte);
				Some(Ok(c))
			},
			
			0b11000000..=0b11011111 => {
				todo!();
			},
			
			0b11100000..=0b11101111 => {
				todo!();
			},
			
			0b11110000..=0b11110111 => {
				todo!();
			},
			
			_ => {
				let err = io::Error::new(InvalidData, "source is not valid UTF-8");
				Some(Err(err))
			},
		}
	}
}
