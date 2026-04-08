# Shopify Ruby Test Harness

This directory contains the Rust-backed replacement `liquid` gem plus the preload shim used to run the upstream Shopify Liquid suite against `liquid-rust`.

The compatibility standard in this repo is intentionally strict: the goal is to pass the same exact Ruby tests used by Shopify Liquid, not to approximate their behavior with a separate hand-written test suite.

To support that harness, this repo ships `liquid_ext`, a Ruby native extension built from [crates/ruby-ext](../crates/ruby-ext). The upstream Shopify Ruby tests run through [harness/bootstrap.rb](bootstrap.rb) and the replacement Ruby gem in [harness/ruby-liquid](ruby-liquid), which load `liquid_ext` and use it to call the hidden conformance entrypoints of the Rust engine.

`ruby-ext` is the bridge layer for the harness only. It preserves the Ruby-facing API shape the Shopify suite expects while executing this Rust implementation underneath. It is not the production API path.

## Baseline

- Ruby: `3.4.1`
- Shopify Liquid commit: `a9c85622ddd784078c2eed34b19a351fe57362cf`
- Primary command: `make harness-test`
- `make harness-test` runs `cargo check --workspace`, force-rebuilds the Ruby extension, then runs the pinned Ruby suite
- The runner prefers the pinned `rbenv` Ruby/Bundler for `RBENV_VERSION=3.4.1` when available, then falls back to `ruby` / `bundle` from `PATH`
- Platform is not pinned in `baseline.yml`; run the harness on the host platform provided by your local machine or CI matrix (for example `arm64-darwin`, `x86_64-darwin`, or `x86_64-linux`)
- If you need multi-platform coverage, configure that in CI or your shell environment directly; there is no harness-specific `PLATFORM` variable to set today
- A green harness run means this fork passed Shopify Liquid's own Ruby tests for that pinned revision

## Layout

- `ruby-liquid/`: replacement gem that mirrors the upstream Ruby API surface
- `bootstrap.rb`: preload shim loaded with `ruby -r`
- `baseline.yml`: pinned environment manifest with the default upstream checkout recorded as the repo-relative sibling path `../shopify-liquid`
- `classifications.yml`: compatibility tracker

## Local Workflow

Rebuild the native extension:

```bash
make harness-compile
```

If your Shopify Liquid checkout is not at `../shopify-liquid`, set `SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid` before running the harness commands below.

Verify the Rust-backed gem loads:

```bash
export SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid
RBENV_VERSION=3.4.1 \
ruby \
-r harness/bootstrap.rb \
-e "require 'liquid'; puts Liquid::RUST_BACKED"
```

Run the full pinned upstream suite:

```bash
export SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid
make harness-test
```

Run a single upstream test file:

```bash
export SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid
make harness-test TEST=test/integration/template_test.rb
```

Show per-test progress explicitly:

```bash
export SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid
make harness-test ARGS="--verbose"
```

Pass extra Ruby test flags through to Minitest:

```bash
export SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid
make harness-test TEST=test/integration/template_test.rb ARGS="--name /include/"
```
