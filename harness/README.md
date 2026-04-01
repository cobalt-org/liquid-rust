# Shopify Ruby Test Harness

This directory hosts the Rust-backed replacement `liquid` gem and the preload shim used to run the upstream Shopify/liquid test suite against `liquid-rust`.

## Layout

- `ruby-liquid/`: replacement gem that mirrors the upstream Ruby API surface
- `bootstrap.rb`: preload shim for `ruby -r`
- `baseline.yml`: pinned environment manifest
- `classifications.yml`: failure tracker

## MVP Commands

```bash
cd /Users/ahmed/Desktop/marfoof/shopify-liquid
ruby -r /Users/ahmed/.codex/worktrees/e06d/liquid-rust/harness/bootstrap.rb -e "require 'liquid'; puts Liquid::RUST_BACKED"
```

```bash
cd /Users/ahmed/Desktop/marfoof/shopify-liquid
ruby -r /Users/ahmed/.codex/worktrees/e06d/liquid-rust/harness/bootstrap.rb test/integration/document_test.rb
```
