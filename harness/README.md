# Shopify Ruby Test Harness

This directory hosts the Rust-backed replacement `liquid` gem and the preload shim used to run the upstream Shopify/liquid test suite against `liquid-rust`.

## Layout

- `ruby-liquid/`: replacement gem that mirrors the upstream Ruby API surface
- `bootstrap.rb`: preload shim for `ruby -r`
- `baseline.yml`: pinned environment manifest
- `classifications.yml`: failure tracker

## MVP Commands

```bash
cd /Users/ahmed/.codex/worktrees/b66f/liquid-rust/harness/ruby-liquid
unset RUSTC_WRAPPER
RB_SYS_CARGO_TARGET_DIR=/tmp/liquid-ruby-ext-target RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/bundle exec rake compile
```

```bash
cd /Users/ahmed/Desktop/marfoof/shopify-liquid
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/ruby -r /Users/ahmed/.codex/worktrees/b66f/liquid-rust/harness/bootstrap.rb -e "require 'liquid'; puts Liquid::RUST_BACKED"
```

```bash
cd /Users/ahmed/Desktop/marfoof/shopify-liquid
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/ruby -Itest -r /Users/ahmed/.codex/worktrees/b66f/liquid-rust/harness/bootstrap.rb test/integration/document_test.rb
```
