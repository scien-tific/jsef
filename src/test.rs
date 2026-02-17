#![cfg(test)]


use super::*;


const OPTS: [ComposeOpts; 4] = [
	ComposeOpts::SIMPLE,
	ComposeOpts::COMPACT,
	ComposeOpts::PRETTY,
	ComposeOpts::PRETTY
		.no_indent()
		.prelude("test\nprelude"),
];


#[test]
fn errors() {
	use JsefErrType::*;
	
	const ERRORS: [(&str, JsefErr); 6] = [
		("0 1 2]",         JsefErr::new(NotEof('1'),              1, 3)),
		("[0 1 2}",        JsefErr::new(Mismatch(']', Some('}')), 1, 7)),
		("\"value",        JsefErr::new(Mismatch('"', None),      1, 7)),
		("{a=0 b.=1 c=2}", JsefErr::new(Unexpected(Some('=')),    1, 8)),
		("{a=0 .b=1 c=2}", JsefErr::new(Mismatch('}', Some('.')), 1, 6)),
		("[0 [1 [2]]]",    JsefErr::new(MaxDepth,                 1, 7)),
	];
	
	for (src, err) in ERRORS {
		let result = parse_value(src).unwrap_err();
		assert_eq!(result, err);
	}
}


#[test]
fn parse() {
	const VAL_PLAIN: &str = "value";
	const VAL_QUOTED: &str = " \"value\" ";
	const DICT: &str = "a=1 b=2 c=3";
	const LIST: &str = "1 2 3";
	const PATH: &str = "a.b.c=value";
	
	let mut dict = JsefDict::default();
	dict.insert("a".to_owned(), JsefValue::string_from("1"));
	dict.insert("b".to_owned(), JsefValue::string_from("2"));
	dict.insert("c".to_owned(), JsefValue::string_from("3"));
	
	let list = JsefList::from([
		JsefValue::string_from("1"),
		JsefValue::string_from("2"),
		JsefValue::string_from("3"),
	]);
	
	let value = JsefValue::string_from("value");
	
	let mut path = JsefDict::default();
	let mut a = JsefDict::default();
	let mut b = JsefDict::default();
	
	b.insert("c".to_owned(), JsefValue::string_from("value"));
	a.insert("b".to_owned(), JsefValue::Dict(b));
	path.insert("a".to_owned(), JsefValue::Dict(a));
	
	let parsed = parse_dict(DICT).unwrap();
	assert_eq!(parsed, dict);
	
	let parsed = parse_list(LIST).unwrap();
	assert_eq!(parsed, list);
	
	let parsed = parse_value(VAL_PLAIN).unwrap();
	assert_eq!(parsed, value);
	
	let parsed = parse_value(VAL_QUOTED).unwrap();
	assert_eq!(parsed, value);
	
	let parsed = parse_dict(PATH).unwrap();
	assert_eq!(parsed, path);
}


#[test]
fn compose() {
	const TARGETS: [&str; 4] = [
		r#"[["0"] "1" "2"] {"path"={"to"="a value"}} "other""#,
		r#"[[0] 1 2] {path.to="a value"} other"#,
		"[\n\t[\n\t\t0\n\t]\n\t1\n\t2\n]\n{\n\tpath.to = \"a value\"\n}\nother",
		"# test\n# prelude\n[ [ 0 ] 1 2 ] { path.to = \"a value\" } other",
	];
	
	let mut root = JsefList::new();
	let mut dict = JsefDict::default();
	let mut path = JsefDict::default();
	
	let list = JsefList::from([
		JsefValue::List(JsefList::from([
			JsefValue::string_from("0"),
		])),
		JsefValue::string_from("1"),
		JsefValue::string_from("2"),
	]);
	
	path.insert("to".to_owned(), JsefValue::string_from("a value"));
	dict.insert("path".to_owned(), JsefValue::Dict(path));
	
	root.push(JsefValue::List(list));
	root.push(JsefValue::Dict(dict));
	root.push(JsefValue::string_from("other"));
	
	let iter = OPTS.iter().zip(TARGETS.iter());
	for (opts, target) in iter {
		let composed = compose_list(&root, opts).unwrap();
		assert_eq!(&composed, target);
	}
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
