# Quickstart: Pure-Rust Production Engine with Ruby-Only Conformance Harness

## Prerequisites

- Rust 1.83.0
- Ruby 3.4.1 and Bundler via `rbenv` to match the current harness script and README
- A local Shopify Liquid checkout at commit `a9c85622ddd784078c2eed34b19a351fe57362cf`
- Optional: `SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid` if the checkout is not in the default sibling path

Note: `harness/baseline.yml` currently records Ruby 3.2.2 while the executable harness tooling uses Ruby 3.4.1. Normalize that metadata drift as part of the implementation so the documented baseline matches the runnable one.

## 1. Validate the production lane without Ruby

Run these from the repository root after the `default-members` change lands:

```bash
cargo check
cargo test
```

Expected result: both commands succeed without Ruby installed and do not build `crates/ruby-ext`.

## 2. Run focused Rust regression coverage

```bash
cargo test --test errors
cargo test --test multithreading
cargo test --test conformance
cargo test -p liquid-lib --test conformance
cargo test --no-default-features --test minimal_parser
```

Use these while iterating on render options, runtime isolation, stdlib scope propagation, and the stdlib-optional `ParserBuilder` contract.

## 3. Run full workspace quality gates in a Ruby-capable environment

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo bench
```

These remain part of merge validation under the project constitution even though benchmark stability is not a formal success criterion in the feature spec.

## 4. Verify the pinned Shopify Liquid baseline

```bash
export SHOPIFY_LIQUID_ROOT=/path/to/shopify-liquid
git -C "$SHOPIFY_LIQUID_ROOT" rev-parse HEAD
```

The printed commit must equal:

```text
a9c85622ddd784078c2eed34b19a351fe57362cf
```

## 5. Run the Ruby conformance harness

```bash
make harness-test
```

The script builds the native extension if needed and runs the pinned upstream tests through the preload shim in `harness/bootstrap.rb`.

## 6. Feature is ready when

- Root `cargo check` and `cargo test` pass without Ruby.
- No conformance-only code compiles in the default build.
- `cargo test --no-default-features --test minimal_parser` proves a minimal parser still works with zero built-in filters and custom Rust extensions.
- `render_with_options` enforces output, render-op, assign-byte, and depth limits.
- Default rendering uses lenient error handling plus Ruby-compatible undefined variable/filter defaults.
- Shared `Template` instances render correctly across threads with isolated per-render state.
- The Ruby harness passes the pinned Shopify Liquid suite for commit `a9c85622ddd784078c2eed34b19a351fe57362cf`.
