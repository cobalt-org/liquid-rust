liquid-rust [![](https://travis-ci.org/cobalt-org/liquid-rust.svg?branch=master)](https://travis-ci.org/cobalt-org/liquid-rust) [![](https://img.shields.io/crates/v/liquid.svg)](https://crates.io/crates/liquid)[![](https://coveralls.io/repos/github/cobalt-org/liquid-rust/badge.svg?branch=master)](https://coveralls.io/github/cobalt-org/liquid-rust?branch=master)
===========

[Liquid templating](http://liquidmarkup.org/) for Rust

Usage
----------

To include liquid in your project add the following to your Cargo.toml:

```toml
[dependencies]
liquid = "0.4"
```

Now you can use the crate in your code
```rust,ignore
extern crate liquid;
```

Example:
```rust
use std::default::Default;
use liquid::Renderable;
use liquid::LiquidOptions;
use liquid::Context;
use liquid::Value;

let options : LiquidOptions = Default::default();
let template = liquid::parse("Liquid! {{num | minus: 2}}", options).unwrap();

let mut data = Context::new();
data.set_val("num", Value::Num(4f32));

let output = template.render(&mut data);
assert_eq!(output.unwrap(), Some("Liquid! 2".to_string()));
```

You can find a reference on Liquid syntax [here](https://github.com/Shopify/liquid/wiki/Liquid-for-Designers).

Plugins
--------
Cache block ( File and Redis ) : https://github.com/FerarDuanSednan/liquid-rust-cache



```rust,skeptic-template
#![allow(unused_imports)] extern crate skeptic; extern crate liquid; fn main() {{ {} }}
```
