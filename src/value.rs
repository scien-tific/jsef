use crate::{JsefList, JsefDict};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsefValue {
	String(String),
	List(JsefList),
	Dict(JsefDict),
}

macro_rules! get {
	( $self:expr, $var:pat => $out:expr ) => {
		match $self {
			$var => Some($out),
			_ => None,
		}
	};
}

macro_rules! take {
	( $self:expr, $var:pat => $out:expr ) => {
		match $self {
			$var => Ok($out),
			other => Err(other),
		}
	};
}

impl JsefValue {
	pub fn new_string() -> Self {
		Self::String(String::new())
	}
	
	pub fn new_list() -> Self {
		Self::List(JsefList::new())
	}
	
	pub fn new_dict() -> Self {
		Self::Dict(JsefDict::default())
	}
	
	
	pub fn string_from<T>(value: T) -> Self
	where T: Into<String> {
		Self::String(value.into())
	}
	
	pub fn list_from<T>(value: T) -> Self
	where T: Into<JsefList> {
		Self::List(value.into())
	}
	
	pub fn dict_from<T>(value: T) -> Self
	where T: Into<JsefDict> {
		Self::Dict(value.into())
	}
	
	
	pub fn is_string(&self) -> bool {
		matches!(self, Self::String(_))
	}
	
	pub fn as_string(&self) -> Option<&String> {
		get!(self, Self::String(s) => s)
	}
	
	pub fn as_string_mut(&mut self) -> Option<&mut String> {
		get!(self, Self::String(s) => s)
	}
	
	pub fn take_string(self) -> Result<String, Self> {
		take!(self, Self::String(s) => s)
	}
	
	
	pub fn is_list(&self) -> bool {
		matches!(self, Self::List(_))
	}
	
	pub fn as_list(&self) -> Option<&JsefList> {
		get!(self, Self::List(l) => l)
	}
	
	pub fn as_list_mut(&mut self) -> Option<&mut JsefList> {
		get!(self, Self::List(l) => l)
	}
	
	pub fn take_list(self) -> Result<JsefList, Self> {
		take!(self, Self::List(l) => l)
	}
	
	
	pub fn is_dict(&self) -> bool {
		matches!(self, Self::Dict(_))
	}
	
	pub fn as_dict(&self) -> Option<&JsefDict> {
		get!(self, Self::Dict(d) => d)
	}
	
	pub fn as_dict_mut(&mut self) -> Option<&mut JsefDict> {
		get!(self, Self::Dict(d) => d)
	}
	
	pub fn take_dict(self) -> Result<JsefDict, Self> {
		take!(self, Self::Dict(d) => d)
	}
}

impl PartialEq<str> for JsefValue {
	fn eq(&self, string: &str) -> bool {
		self.as_string().is_some_and(|s| s == string)
	}
}

impl PartialEq<JsefValue> for str {
	fn eq(&self, value: &JsefValue) -> bool {
		value.as_string().is_some_and(|s| self == s)
	}
}

impl PartialEq<String> for JsefValue {
	fn eq(&self, string: &String) -> bool {
		self.as_string().is_some_and(|s| s == string)
	}
}

impl PartialEq<JsefValue> for String {
	fn eq(&self, value: &JsefValue) -> bool {
		value.as_string().is_some_and(|s| self == s)
	}
}

impl PartialEq<JsefList> for JsefValue {
	fn eq(&self, list: &JsefList) -> bool {
		self.as_list().is_some_and(|l| l == list)
	}
}

impl PartialEq<JsefValue> for JsefList {
	fn eq(&self, value: &JsefValue) -> bool {
		value.as_list().is_some_and(|l| self == l)
	}
}

impl PartialEq<JsefDict> for JsefValue {
	fn eq(&self, dict: &JsefDict) -> bool {
		self.as_dict().is_some_and(|d| d == dict)
	}
}

impl PartialEq<JsefValue> for JsefDict {
	fn eq(&self, value: &JsefValue) -> bool {
		value.as_dict().is_some_and(|d| self == d)
	}
}
