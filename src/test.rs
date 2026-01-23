#![cfg(test)]

// Be warned: I am not good at writing tests.


use super::*;


const DICT_STR: &str = r##"
key = value
list = [0 1 2 3]
0 = 1 # same as "0" = "1"

# Special characters and escape sequences need quoted strings
"#" = "multiline\nvalue"

dict = {
	a = x
	b = y
}

# or the same with path notation...
dict.a = x
dict.b = y
dict.a.oops = z # dict.a is now replaced with {oops = z}
"##;


#[test]
fn test_parse() {
	let parsed = parse_dict(DICT_STR).unwrap();
	let mut expected = JsefDict::default();
	let mut list = JsefList::new();
	let mut dict = JsefDict::default();
	let mut a = JsefDict::default();
	
	list.push(JsefValue::from("0"));
	list.push(JsefValue::from("1"));
	list.push(JsefValue::from("2"));
	list.push(JsefValue::from("3"));
	
	a.insert("oops".to_owned(), JsefValue::from("z"));
	dict.insert("a".to_owned(), JsefValue::Dict(a));
	dict.insert("b".to_owned(), JsefValue::from("y"));
	
	expected.insert("key".to_owned(), JsefValue::from("value"));
	expected.insert("list".to_owned(), JsefValue::List(list));
	expected.insert("0".to_owned(), JsefValue::from("1"));
	expected.insert("#".to_owned(), JsefValue::from("multiline\nvalue"));
	expected.insert("dict".to_owned(), JsefValue::Dict(dict));
	
	assert_eq!(parsed, expected);
}


#[test]
fn test_compose() {
	let dict = parse_dict(DICT_STR).unwrap();
	
	let composed = compose_dict(&dict, &ComposeOpts::SIMPLE).unwrap();
	let parsed = parse_dict(&composed).unwrap();
	assert_eq!(dict, parsed);
	
	let composed = compose_dict(&dict, &ComposeOpts::COMPACT).unwrap();
	let parsed = parse_dict(&composed).unwrap();
	assert_eq!(dict, parsed);
	
	let composed = compose_dict(&dict, &ComposeOpts::PRETTY).unwrap();
	let parsed = parse_dict(&composed).unwrap();
	assert_eq!(dict, parsed);
}
