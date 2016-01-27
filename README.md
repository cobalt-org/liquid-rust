liquid-rust [![](https://travis-ci.org/cobalt-org/liquid-rust.svg?branch=master)](https://travis-ci.org/cobalt-org/liquid-rust) [![](https://img.shields.io/crates/v/liquid.svg)](https://crates.io/crates/liquid)
===========

[Liquid templating](http://liquidmarkup.org/) for Rust

Usage
----------

To include liquid in your project add the following to your Cargo.toml:

```toml
[dependencies]
liquid = "0.3"
```

Now you can use the crate in your code
```rust
extern crate liquid;
```
Example:
```rust
let mut text = String::new();
File::open("./tests/simple/template.txt").unwrap().read_to_string(&mut text);
let mut options : LiquidOptions = Default::default();
let template = parse(&text, &mut options).unwrap();

let mut data = Context::new();
data.set_val("num", Value::Num(5f32));
data.set_val("numTwo", Value::Num(6f32));

let output = template.render(&mut data);
```

Plugins
--------
Cache block ( File and Redis ) : https://github.com/FerarDuanSednan/liquid-rust-cache

TODO
---------

Standard Filters

- [ ] *date* - reformat a date (syntax reference)
- [ ] *capitalize* - capitalize words in the input sentence
- [ ] *downcase* - convert an input string to lowercase
- [x] *upcase* - convert an input string to uppercase
- [ ] *first* - get the first element of the passed in array
- [ ] *last* - get the last element of the passed in array
- [ ] *join* - join elements of the array with certain character between them
- [ ] *sort* - sort elements of the array
- [ ] *map* - map/collect an array on a given property
- [x] *size* - return the size of an array or string
- [ ] *escape* - escape a string
- [ ] *escape_once* - returns an escaped version of html without affecting existing escaped entities
- [ ] *strip_html* - strip html from string
- [ ] *strip_newlines* - strip all newlines (\n) from string
- [ ] *newline_to_br* - replace each newline (\n) with html break
- [x] *replace* - replace each occurrence e.g. {{ 'foofoo' | replace:'foo','bar' }} #=> 'barbar'
- [ ] *replace_first* - replace the first occurrence e.g. {{ 'barbar' | replace_first:'bar','foo' }} #=> 'foobar'
- [ ] *remove* - remove each occurrence e.g. {{ 'foobarfoobar' | remove:'foo' }} #=> 'barbar'
- [ ] *remove_first* - remove the first occurrence e.g. {{ 'barbar' | remove_first:'bar' }} #=> 'bar'
- [ ] *truncate* - truncate a string down to x characters. It also accepts a second parameter that will append to the string e.g. {{ 'foobarfoobar' | truncate: 5, '.' }} #=> 'foob.'
- [ ] *truncatewords* - truncate a string down to x words
- [ ] *prepend* - prepend a string e.g. {{ 'bar' | prepend:'foo' }} #=> 'foobar'
- [ ] *pluralize* - return the second word if the input is not 1, otherwise return the first word e.g. {{ 3 | pluralize: 'item', 'items' }} #=> 'items'
- [ ] *append* - append a string e.g. {{ 'foo' | append:'bar' }} #=> 'foobar'
- [ ] *slice* - slice a string. Takes an offset and length, e.g. {{ "hello" | slice: -3, 3 }} #=> llo
- [x] *minus* - subtraction e.g. {{ 4 | minus:2 }} #=> 2
- [ ] *plus* - addition e.g. {{ '1' | plus:'1' }} #=> 2, {{ 1 | plus:1 }} #=> 2
- [ ] *times* - multiplication e.g {{ 5 | times:4 }} #=> 20
- [ ] *divided_by* - integer division e.g. {{ 10 | divided_by:3 }} #=> 3
- [ ] *round* - rounds input to the nearest integer or specified number of decimals
- [ ] *split* - split a string on a matching pattern e.g. {{ "a~b" | split:"~" }} #=> ['a','b']
- [ ] *modulo* - remainder, e.g. {{ 3 | modulo:2 }} #=> 1
- [ ] *reverse* - reverse sort the passed in array
