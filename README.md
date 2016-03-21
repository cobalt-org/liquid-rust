liquid-rust [![](https://travis-ci.org/cobalt-org/liquid-rust.svg?branch=master)](https://travis-ci.org/cobalt-org/liquid-rust) [![](https://img.shields.io/crates/v/liquid.svg)](https://crates.io/crates/liquid)[![](https://coveralls.io/repos/github/cobalt-org/liquid-rust/badge.svg?branch=master)](https://coveralls.io/github/cobalt-org/liquid-rust?branch=master)
===========

[Liquid templating](http://liquidmarkup.org/) for Rust

Usage
----------

To include liquid in your project add the following to your Cargo.toml:

```toml
[dependencies]
liquid = "0.5"
```

Now you can use the crate in your code
```
extern crate liquid;
```

Example:
```rust
use liquid::{Renderable, Context, Value};

let template = liquid::parse("Liquid! {{num | minus: 2}}", Default::default()).unwrap();

let mut context = Context::new();
context.set_val("num", Value::Num(4f32));

let output = template.render(&mut context);
assert_eq!(output.unwrap(), Some("Liquid! 2".to_string()));
```

You can find a reference on Liquid syntax [here](https://github.com/Shopify/liquid/wiki/Liquid-for-Designers).

Plugins
--------
Cache block ( File and Redis ) : https://github.com/FerarDuanSednan/liquid-rust-cache

Extending Liquid
--------

### Create your own filters

Creating your own filters is very easy. Filters are simply functions or
closures that take an input `Value` and a `Vec<Value>` of optional arguments
and return a `Value` to be rendered or consumed by chained filters.

```rust
use liquid::{Renderable, Context, Value, FilterError};

let template = liquid::parse("{{'hello' | shout}}", Default::default()).unwrap();

let mut context = Context::new();

// create our custom shout filter
context.add_filter("shout", Box::new(|input, _args| {
    if let &Value::Str(ref s) = input {
      Ok(Value::Str(s.to_uppercase()))
    } else {
      Err(FilterError::InvalidType("Expected a string".to_owned()))
    }
}));

let output = template.render(&mut context);
assert_eq!(output.unwrap(), Some("HELLO".to_owned()));
```

### Create your own tags

Tags are made up of two parts, the initialization and the rendering.

Initialization happens when the parser hits a Liquid tag that has your
designated name. You will have to specify a function or closure that will
then return a `Renderable` object to do the rendering.

```rust
use liquid::{LiquidOptions, Renderable, Context, Error};

// our renderable object
struct Shout {
    text: String
}
impl Renderable for Shout {
    fn render(&self, _context: &mut Context) -> Result<Option<String>, Error>{
        Ok(Some(self.text.to_uppercase()))
    }
}

let mut options : LiquidOptions = Default::default();

// initialize the tag and pass a closure that will return a new Shout renderable
options.tags.insert("shout".to_owned(), Box::new(|_tag_name, arguments, _options| {
    Box::new(Shout{text: arguments[0].to_string()})
}));

// use our new tag
let template = liquid::parse("{{shout 'hello'}}", options).unwrap();

let mut context = Context::new();
let output = template.render(&mut context);
assert_eq!(output.unwrap(), Some("HELLO".to_owned()));
```

### Create your own tag blocks

Blocks work very similar to Tags. The only difference is that blocks contain other
markup, which is why block initialization functions take another argument, a list
of `Element`s that are inside the specified block.

For an implementation of a `Shout` block, see [this example](https://github.com/johannhof/liquid-plugin-example/blob/master/src/lib.rs).

----------

Ignore this:
```rust,skeptic-template
extern crate skeptic; extern crate liquid; fn main() {{ {} }}
```
