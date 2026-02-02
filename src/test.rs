#![cfg(test)]


use super::*;


const OPTS: [ComposeOpts; 4] = [
	ComposeOpts::SIMPLE,
	ComposeOpts::COMPACT,
	ComposeOpts::PRETTY,
	ComposeOpts {
		indent: None,
		force_quotes: false,
		dense: false,
		fold_dicts: true,
		prelude: Some("test\nprelude"),
	},
];


#[test]
fn errors() {
	use JsefErrType::*;
	const SOURCES: [(&str, JsefErr); 13] = [
		("{a=1 b=2 c=3",          JsefErr::new(Mismatch('}', None),      1, 13)),
		("a=1 b=2 c=3}",          JsefErr::new(NotEof('='),              1,  2)),
		("[1 2 3",                JsefErr::new(Mismatch(']', None),      1,  7)),
		("1 2 3]",                JsefErr::new(NotEof('2'),              1,  3)),
		("{a=1 \"b=2 c=3}",       JsefErr::new(Mismatch('"', None),      1, 15)),
		("{a=1 b\"=2 c=3}",       JsefErr::new(Mismatch('=', Some('"')), 1,  7)),
		("{a=1 b= c=3}",          JsefErr::new(Mismatch('}', Some('=')), 1, 10)),
		("{a=1 =1 c=3}",          JsefErr::new(Mismatch('}', Some('=')), 1,  6)),
		("{a=1 b c=3}",           JsefErr::new(Mismatch('=', Some('c')), 1,  8)),
		("[1 b=2 3]",             JsefErr::new(Mismatch(']', Some('=')), 1,  5)),
		("{a=1 new\nline=2 c=3}", JsefErr::new(Mismatch('=', Some('l')), 2,  1)),
		("{a=1 b.=2 c=3}",        JsefErr::new(Unexpected(Some('=')),    1,  8)),
		("{a=1 .b=2 c=3}",        JsefErr::new(Mismatch('}', Some('.')), 1,  6)),
	];
	
	for (string, err) in SOURCES {
		let result = parse_value(string).unwrap_err();
		assert_eq!(result, err);
	}
}


#[test]
fn parse_values() {
	const PLAIN: &str = "value";
	const QUOTED: &str = " \"value\" ";
	
	let value = JsefValue::string_from("value");
	
	let parsed = parse_value(PLAIN).unwrap();
	assert_eq!(parsed, value);
	
	let parsed = parse_value(QUOTED).unwrap();
	assert_eq!(parsed, value);
}


#[test]
fn dict_parse() {
	const SOURCE: &str = "a=1 b=2 c=3";
	
	let mut root = JsefDict::default();
	root.insert("a".to_owned(), JsefValue::string_from("1"));
	root.insert("b".to_owned(), JsefValue::string_from("2"));
	root.insert("c".to_owned(), JsefValue::string_from("3"));
	
	let parsed = parse_dict(SOURCE).unwrap();
	assert_eq!(parsed, root);
}


#[test]
fn list_parse() {
	const SOURCE: &str = "[1] 2 3";
	
	let list = JsefList::from([JsefValue::string_from("1")]);
	let root = JsefList::from([
		JsefValue::List(list),
		JsefValue::string_from("2"),
		JsefValue::string_from("3"),
	]);
	
	let parsed = parse_list(SOURCE).unwrap();
	assert_eq!(parsed, root);
}


#[test]
fn path_parse() {
	const SOURCE: &str = "a.b.c=value";
	
	let mut root = JsefDict::default();
	let mut a = JsefDict::default();
	let mut b = JsefDict::default();
	
	b.insert("c".to_owned(), JsefValue::string_from("value"));
	a.insert("b".to_owned(), JsefValue::Dict(b));
	root.insert("a".to_owned(), JsefValue::Dict(a));
	
	let parsed = parse_dict(SOURCE).unwrap();
	assert_eq!(parsed, root);
}


#[test]
fn stresstest() {
	const SOURCE: &str = r##"
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
		dict.a . oops = z # dict.a is now replaced with {oops = z}
	"##;
	
	let mut root = JsefDict::default();
	let mut dict = JsefDict::default();
	let mut a = JsefDict::default();
	
	let list = JsefValue::list_from([
		JsefValue::string_from("0"),
		JsefValue::string_from("1"),
		JsefValue::string_from("2"),
		JsefValue::string_from("3"),
	]);
	
	a.insert("oops".to_owned(), JsefValue::string_from("z"));
	dict.insert("a".to_owned(), JsefValue::Dict(a));
	dict.insert("b".to_owned(), JsefValue::string_from("y"));
	
	root.insert("key".to_owned(), JsefValue::string_from("value"));
	root.insert("list".to_owned(), list);
	root.insert("0".to_owned(), JsefValue::string_from("1"));
	root.insert("#".to_owned(), JsefValue::string_from("multiline\nvalue"));
	root.insert("dict".to_owned(), JsefValue::Dict(dict));
	
	let parsed = parse_dict(SOURCE).unwrap();
	assert_eq!(parsed, root);
	
	for opts in OPTS.iter() {
		let composed = compose_dict(&root, opts).unwrap();
		let parsed = parse_dict(&composed).unwrap();
		assert_eq!(parsed, root);
	}
}


#[test]
fn compose() {
	const TARGETS: [&str; 4] = [
		r#"{"path"={"to"="a value"}} "other" {"a"="0" "b"="1"}"#,
		r#"{path.to="a value"} other {a=0 b=1}"#,
		"{\n\tpath.to = \"a value\"\n}\nother\n{\n\ta = 0\n\tb = 1\n}",
		"# test\n# prelude\n{ path.to = \"a value\" } other { a = 0 b = 1 }",
	];
	
	let mut root = JsefList::new();
	let mut left = JsefDict::default();
	let mut right = JsefDict::default();
	let mut path = JsefDict::default();
	
	path.insert("to".to_owned(), JsefValue::string_from("a value"));
	left.insert("path".to_owned(), JsefValue::Dict(path));
	right.insert("a".to_owned(), JsefValue::string_from("0"));
	right.insert("b".to_owned(), JsefValue::string_from("1"));
	root.push(JsefValue::Dict(left));
	root.push(JsefValue::string_from("other"));
	root.push(JsefValue::Dict(right));
	
	let iter = OPTS.iter().zip(TARGETS.iter());
	for (opts, target) in iter {
		let composed = compose_list(&root, opts).unwrap();
		assert_eq!(&composed, target);
	}
}
