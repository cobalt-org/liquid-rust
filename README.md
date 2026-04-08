liquid-rust
===========

> [Liquid templating](https://shopify.github.io/liquid/) for Rust

[![Crates Status](https://img.shields.io/crates/v/liquid.svg)](https://crates.io/crates/liquid)

This repository is a fork of [`cobalt-org/liquid-rust`](https://github.com/cobalt-org/liquid-rust).

## Why This Fork Exists

The main difference between this fork and the upstream `cobalt-org/liquid-rust` repository is how seriously Shopify Liquid compatibility is treated here.

This fork aims to be a **100% compatibility port of Shopify Liquid** for the pinned upstream revision:

- the target behavior is Shopify Liquid, not just “Liquid-like” behavior
- compatibility is validated by running the **same exact Ruby tests** used by Shopify Liquid
- failures against that upstream suite are treated as compatibility bugs to fix

We still support two separate runtime goals that need different behavior:

- Production rendering should be pure Rust, with no Ruby dependency in the default build.
- Production rendering should use practical defaults: missing variables become `nil`, unknown filters pass input through, and render behavior is configurable per call with `RenderOptions`.
- Shopify compatibility still matters, so the repo includes a hidden conformance path and Ruby harness that runs the upstream Shopify Liquid suite against this implementation.

In short: production stays pure Rust, but the compatibility bar is defined by Shopify Liquid itself, and we verify that by running Shopify's own Ruby tests against this fork.

## How To Use

### 1. Normal Rust usage

Add the crate:

```console
$ cargo add liquid
```

Parse and render in pure Rust:

```rust
let template = liquid::ParserBuilder::with_stdlib()
    .build().unwrap()
    .parse("Liquid! {{ num | minus: 2 }}").unwrap();

let globals = liquid::object!({
    "num": 4f64
});

let output = template.render(&globals).unwrap();
assert_eq!(output, "Liquid! 2");
```

Default rendering behavior:

- missing variables resolve to `nil`
- unknown filters pass the input through unchanged
- `Template::render` and `Template::render_to` stay on the pure-Rust path

Use `RenderOptions` when you need stricter behavior or hard limits:

```rust
use liquid::{ErrorMode, RenderOptions};

let result = template
    .render_with_options(
        &globals,
        &RenderOptions {
            max_output_bytes: Some(1_000_000),
            max_render_ops: Some(100_000),
            max_assign_bytes: Some(500_000),
            max_depth: Some(100),
            strict_variables: true,
            strict_filters: true,
            error_mode: ErrorMode::Strict,
        },
    )
    .unwrap();

assert_eq!(result.output, "Liquid! 2");
assert!(result.errors.is_empty());
```

See [docs/RENDER_OPTIONS_GUIDE.md](docs/RENDER_OPTIONS_GUIDE.md) for the full API.

### 2. Minimal parser usage

If you want a parser with no built-in stdlib filters:

```toml
[dependencies]
liquid = { version = "1.0.0", default-features = false }
```

Then register only your own tags, blocks, and filters.

### 3. Shopify compatibility verification

The Ruby conformance harness is not part of the production API. Its purpose is to prove Shopify Liquid compatibility by running the exact upstream Ruby test suite against this fork.

This is intentionally strict: the goal is not to approximate Shopify behavior with a separate hand-written test suite. The goal is to pass Shopify Liquid's own Ruby tests for the pinned upstream revision.

To support that harness, this repo includes `liquid_ext`, a Ruby native extension built from [crates/ruby-ext](crates/ruby-ext). The upstream Shopify Ruby tests do not call the pure Rust API directly. Instead, they run through the preload shim in [harness/bootstrap.rb](harness/bootstrap.rb) and the replacement Ruby gem in [harness/ruby-liquid](harness/ruby-liquid), which load `liquid_ext` and use it to invoke this implementation through the hidden conformance entrypoints.

That extension exists only for compatibility verification. It adapts the Ruby test environment to the Rust engine, preserves the Ruby-facing API shape the Shopify suite expects, and keeps the production API path pure Rust.

Prerequisites:

- Ruby `3.4.1`
- a local checkout of Shopify Liquid at commit `a9c85622ddd784078c2eed34b19a351fe57362cf`

Compile the Ruby extension:

```bash
make harness-compile
```

Run the full pinned upstream suite:

```bash
export SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid
make harness-test
```

Run one upstream test file:

```bash
export SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid
make harness-test TEST=test/integration/template_test.rb
```

See [harness/README.md](harness/README.md) for the harness layout and workflow.

Goals:
1. Conformant. This fork aims for 100% compatibility with the pinned Shopify Liquid revision, and incompatibilities with [strict shopify/liquid][shopify-liquid] are [bugs to be fixed][shopify-compat].
2. Flexible. Liquid embraces [variants][liquid-variants] for different domains and we want to follow in that spirit.
3. Performant. Do the best we can within what is conformant.

[shopify-liquid]: https://github.com/Shopify/liquid
[shopify-compat]: https://github.com/cobalt-org/liquid-rust/labels/shopify-compatibility
[liquid-variants]: https://shopify.github.io/liquid/basics/variations/

Example applications using liquid-rust:
- [cobalt]: static site generator.
- [cargo-tarball]: crate bin packaging tool.
- [cargo-generate]: crate generator from templates.
- [Mandy]: A hypersonic, easy-to-use, performant static-site generator.

[cobalt]: https://cobalt-org.github.io/
[cargo-tarball]: https://github.com/crate-ci/cargo-tarball
[cargo-generate]: https://github.com/ashleygwilliams/cargo-generate
[Mandy]: https://github.com/alyxshang/mandy

You can find a reference on Liquid syntax [here](https://github.com/Shopify/liquid/wiki/Liquid-for-Designers).

Customizing Liquid
------------------

### Language Variants

By default, `liquid-rust` has no filters, tags, or blocks.  You can enable the
default set or pick and choose which to add to suit your application.

### Create your own filters

Creating your own filters is very easy. Filters are simply functions or
closures that take an input `Value` and a `Vec<Value>` of optional arguments
and return a `Value` to be rendered or consumed by chained filters.

See
[filters/](https://github.com/cobalt-org/liquid-rust/blob/master/crates/lib/src/stdlib/filters)
for what a filter implementation looks like.  You can then register it by
calling `liquid::ParserBuilder::filter`.

### Create your own tags

Tags are made up of two parts, the initialization and the rendering.

Initialization happens when the parser hits a Liquid tag that has your
designated name. You will have to specify a function or closure that will
then return a `Renderable` object to do the rendering.

See
[include_tag.rs](https://github.com/cobalt-org/liquid-rust/blob/master/crates/lib/src/stdlib/tags/include_tag.rs)
for what a tag implementation looks like.  You can then register it by calling `liquid::ParserBuilder::tag`.

### Create your own tag blocks

Blocks work very similar to Tags. The only difference is that blocks contain other
markup, which is why block initialization functions take another argument, a list
of `Element`s that are inside the specified block.

See
[comment_block.rs](https://github.com/cobalt-org/liquid-rust/blob/master/crates/lib/src/stdlib/blocks/comment_block.rs)
for what a block implementation looks like.  You can then register it by
calling `liquid::ParserBuilder::block`.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/license/mit>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual-licensed as above, without any additional terms or
conditions.
