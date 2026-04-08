# Implementation Plan: Pure-Rust Production Engine with Ruby-Only Conformance Harness

**Branch**: `001-prod-conformance-split` | **Date**: 2026-04-04 | **Spec**: `specs/001-prod-conformance-split/spec.md`
**Input**: Feature specification from `specs/001-prod-conformance-split/spec.md`

## Summary

Deliver a single refactor that separates the production engine from Ruby conformance support. Default builds become pure Rust and validate through root `cargo check` and `cargo test` using Cargo `default-members`, while feature-gated conformance builds preserve the hidden bridge needed by `crates/ruby-ext` to pass the pinned Shopify Liquid suite at commit `a9c85622ddd784078c2eed34b19a351fe57362cf`. The implementation centers on a sealed render-policy abstraction, executor-owned limit and error handling, new public render configuration/result types, and selective propagation of shared per-render state through stack frames and isolated render scopes.

## Technical Context

**Language/Version**: Rust edition 2021 with MSRV 1.83.0; Ruby 3.4.1 for harness execution to match the current harness script and README  
**Primary Dependencies**: Workspace crates `liquid`, `liquid-core`, `liquid-lib`, `liquid-derive`, and `liquid_ext`; `anymap2`, `pest`, `regex`, `time`, `kstring`, `serde`; `magnus` and `rb-sys` only in `crates/ruby-ext`  
**Storage**: N/A for product data; in-memory render state plus filesystem-backed harness checkout/fixtures  
**Testing**: Root `cargo check` and `cargo test` with `default-members`; `cargo test --no-default-features --test minimal_parser` for the stdlib-optional contract; `cargo test --workspace` in a Ruby-capable environment; `tests/`, `tests/conformance_ruby/`, `crates/lib/tests/`; `make harness-test`; `cargo bench`; `cargo clippy --workspace --all-targets`; `cargo fmt --check`  
**Target Platform**: Tier-1 Rust targets for the library crates; Ruby-capable macOS/Linux dev and CI environments for the conformance harness against Shopify Liquid commit `a9c85622ddd784078c2eed34b19a351fe57362cf`  
**Project Type**: Multi-crate Rust library workspace with an internal Ruby native-extension harness  
**Performance Goals**: Preserve render/parsing benchmark coverage, keep production per-render policy state single-threaded with no `Arc<Mutex<_>>` on the hot path, and keep output-byte enforcement hard and mid-write  
**Constraints**: Default builds must compile without Ruby, `magnus`, or conformance-only code; `conformance-harness` gates all conformance-only behavior; public render APIs remain non-generic; `Template` stays shareable across threads; breaking-release target is `1.0.0` to satisfy the constitution's major-bump rule; benchmark execution remains a merge requirement even though it is not a formal success criterion; harness metadata drift (`baseline.yml` vs script default Ruby version) must be normalized during implementation  
**Scale/Scope**: One large refactor spanning the root facade, `liquid-core` runtime/parser internals, `liquid-lib` stdlib tags and blocks, `crates/ruby-ext`, harness scripts/manifests, and both Rust and Ruby conformance tests

## Constitution Check

### Pre-Phase 0 Gate Review

- **Conformance First**: PASS. The plan keeps 100% pass of the pinned Shopify Liquid suite as the compatibility bar and keeps Ruby-visible behavior under the harness.
- **Extensibility**: PASS. Public `ParserBuilder`-based registration for Rust filters, tags, and blocks remains the stable production extension surface, and the plan now includes explicit stdlib-disabled validation for the minimal-parser contract.
- **Crate Modularity**: PASS. The design works inside the existing workspace crates; no new organizational crate is required.
- **Performance Discipline**: PASS. Benchmarks remain required before merge, and the design explicitly avoids new cross-thread synchronization on production render hot paths.
- **Code Quality**: PASS. Workspace lints, MSRV, and dual-license constraints remain unchanged. The breaking API now targets `1.0.0`, satisfying the constitution's major-version requirement.
- **Development Workflow**: PASS. The feature adds a Ruby-free production lane without removing full-workspace, lint, formatting, benchmark, and Ruby-harness validation before merge.

### Post-Phase 1 Design Re-Check

- **Conformance First**: PASS. `contracts/conformance-bridge.md` and `quickstart.md` keep the pinned commit and harness execution as explicit validation steps.
- **Extensibility**: PASS. `contracts/public-render-api.md` preserves production-facing Rust extension points, keeps Ruby-only behavior behind the conformance bridge, and now makes the stdlib-optional minimal-parser guarantee explicit.
- **Crate Modularity**: PASS. `data-model.md` and the project structure keep responsibilities split across the root crate, `liquid-core`, `liquid-lib`, and `crates/ruby-ext`.
- **Performance Discipline**: PASS. `research.md` records benchmark review as a merge requirement even though benchmark stability is no longer a formal success criterion in the feature spec.
- **Code Quality**: PASS. The design artifacts keep public API changes explicit, avoid unresolved clarifications, and do not require an MSRV or lint-policy change.
- **Development Workflow**: PASS. `quickstart.md` includes the production lane, full workspace checks, and the Ruby harness lane needed to satisfy the constitution.

**Gate Result**: PASS

## Project Structure

### Documentation (this feature)

```text
specs/001-prod-conformance-split/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── conformance-bridge.md
│   └── public-render-api.md
└── tasks.md
```

### Source Code (repository root)

```text
./
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── parser.rs
│   └── template.rs
├── crates/
│   ├── core/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser/
│   │       │   ├── filter_chain.rs
│   │       │   ├── mod.rs
│   │       │   └── registry.rs
│   │       └── runtime/
│   │           ├── mod.rs
│   │           ├── runtime.rs
│   │           ├── stack.rs
│   │           └── template.rs
│   ├── lib/
│   │   └── src/
│   │       └── stdlib/
│   │           ├── blocks/
│   │           │   ├── capture_block.rs
│   │           │   └── for_block.rs
│   │           └── tags/
│   │               ├── assign_tag.rs
│   │               ├── include_tag.rs
│   │               └── render_tag.rs
│   └── ruby-ext/
│       └── src/
│           ├── callbacks.rs
│           ├── context.rs
│           ├── environment.rs
│           ├── lib.rs
│           └── template.rs
├── harness/
│   ├── baseline.yml
│   ├── bootstrap.rb
│   └── ruby-liquid/
├── scripts/
│   └── harness/
│       └── run_shopify_liquid_harness_tests.sh
└── tests/
    ├── conformance.rs
    ├── conformance_ruby/
    ├── errors.rs
    └── multithreading.rs
```

**Structure Decision**: Keep the existing multi-crate workspace. Implement the production/conformance split inside the current root facade, `liquid-core`, `liquid-lib`, and `crates/ruby-ext` rather than introducing a new crate or a second workspace. Use the spec directory only for planning artifacts.

## Implementation Phases

1. **Workspace and build boundary**: Add production-only `default-members`, introduce the `conformance-harness` feature path through the Rust crates, and expose the hidden conformance module required by `ruby-ext`.
2. **Public render API**: Add `RenderOptions`, `ErrorMode`, `RenderOutput`, default lenient entrypoints, and the breaking-change removal path for `Template::render_to_runtime`.
3. **Runtime executor and policy refactor**: Shrink `Runtime`, move error/limit handling to the executor and register-carried policies, and enforce cumulative production limits.
4. **Scope, filter, and conformance integration**: Propagate shared policy/adapter state through `SandboxedStackFrame`, update filter dispatch in `filter_chain.rs`, and keep Ruby-only late registration, resource limits, and live-scope behavior behind the feature gate.
5. **Validation and release readiness**: Extend Rust tests, rerun the stdlib-disabled minimal-parser validation plus workspace quality gates, execute the Ruby harness against commit `a9c85622ddd784078c2eed34b19a351fe57362cf`, and prepare the `1.0.0` release notes for the breaking API change.

## Complexity Tracking

No constitution violations require special justification.
