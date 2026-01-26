# JSeF: Just a Serialization Format

A basic, JSON-like format with super fancy features such as:
- Omittable root-level brackets (sometimes)
- Omittable double quotes (sometimes)
- Paths
- Line comments
- Only string values, parse them yourself!

## Example

```text
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
```

## TODO

- [x] `ComposeOpts::fold_dicts`
- [x] `ComposeOpts::prelude`
- [ ] `io::Read` and `io::Write` based parsing and composing
- [ ] `JsefList` and `JsefDict` as wrapper structs
