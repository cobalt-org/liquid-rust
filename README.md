liquid-rust [![Travis Status](https://travis-ci.org/cobalt-org/liquid-rust.svg?branch=master)](https://travis-ci.org/cobalt-org/liquid-rust) [![Appveyor Status](https://ci.appveyor.com/api/projects/status/n1nqaitd5uja8tsi/branch/master?svg=true)](https://ci.appveyor.com/project/johannhof/liquid-rust/branch/master) [![Crates Status](https://img.shields.io/crates/v/liquid.svg)](https://crates.io/crates/liquid) [![Coverage Status](https://coveralls.io/repos/github/cobalt-org/liquid-rust/badge.svg?branch=master)](https://coveralls.io/github/cobalt-org/liquid-rust?branch=master) [![Dependency Status](https://dependencyci.com/github/cobalt-org/liquid-rust/badge)](https://dependencyci.com/github/cobalt-org/liquid-rust)
===========

[Liquid templating](http://liquidmarkup.org/) for Rust

Usage
----------

To include liquid in your project add the following to your Cargo.toml:

```toml
[dependencies]
liquid = "0.13"
```

Now you can use the crate in your code
```rust
extern crate liquid;
```

Example:
```rust
let template = liquid::ParserBuilder::with_liquid()
    .build()
    .parse("Liquid! {{num | minus: 2}}").unwrap();

let mut globals = liquid::Object::new();
globals.insert("num".to_owned(), liquid::Value::scalar(4f32));

let output = template.render(&globals).unwrap();
assert_eq!(output, "Liquid! 2".to_string());
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

See
[filters.rs](https://github.com/cobalt-org/liquid-rust/blob/master/src/filters.rs)
for what a filter implementation looks like.  You can then register it by
calling `liquid::ParserBuilder::filter`.

### Create your own tags

Tags are made up of two parts, the initialization and the rendering.

Initialization happens when the parser hits a Liquid tag that has your
designated name. You will have to specify a function or closure that will
then return a `Renderable` object to do the rendering.

See
[include_tag.rs](https://github.com/cobalt-org/liquid-rust/blob/master/src/tags/include_tag.rs)
for what a tag implementation looks like.  You can then register it by calling `liquid::ParserBuilder::tag`.

### Create your own tag blocks

Blocks work very similar to Tags. The only difference is that blocks contain other
markup, which is why block initialization functions take another argument, a list
of `Element`s that are inside the specified block.

See
[comment_block.rs](https://github.com/cobalt-org/liquid-rust/blob/master/src/tags/comment_block.rs)
for what a block implementation looks like.  You can then register it by
calling `liquid::ParserBuilder::block`.

----------

<!---

Skeptic template:
```rust,skeptic-template
extern crate skeptic; extern crate liquid; fn main() {{ {} }}
```

-->
