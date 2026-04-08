# Claude Review Context

## Purpose

This file is context for reviewing the architectural plan in:

- [PRODUCTION_CONFORMANCE_SPLIT_SPEC.md](./PRODUCTION_CONFORMANCE_SPLIT_SPEC.md)

The goal of the review is to find plan flaws, ambiguities, migration risks, or places where the proposed architecture still leaks Ruby-specific semantics back into production.

This is not a request to implement the plan yet.

## Current Project State

- The repository is `liquid-rust`.
- The current branch contains a large compatibility effort to run the upstream Shopify Ruby Liquid suite against a Rust-backed implementation.
- The Ruby harness is currently green on this branch.
- The next step is architectural cleanup: keep the compatibility signal, but remove Ruby-driven complexity from production design.

## Intended Direction

- Production should become a pure Rust Liquid engine.
- Ruby should be used only as a full conformance harness.
- The production/public API should shrink rather than preserve Ruby-oriented runtime hooks.
- The Ruby adapter and harness may remain in-repo, but they should not shape the production architecture.

## Important Existing Docs

- [PRODUCTION_CONFORMANCE_SPLIT_SPEC.md](./PRODUCTION_CONFORMANCE_SPLIT_SPEC.md)
  - the main spec being reviewed
- [SHOPIFY_RUBY_TEST_HARNESS_PLAN.md](./SHOPIFY_RUBY_TEST_HARNESS_PLAN.md)
  - earlier plan for getting the upstream Shopify Ruby suite running against the Rust-backed harness
- [RENDER_SESSION_BRIDGE_PLAN.md](./RENDER_SESSION_BRIDGE_PLAN.md)
  - earlier narrower plan for live render-scope/session bridging into Ruby callbacks

Those older docs provide useful history, but `PRODUCTION_CONFORMANCE_SPLIT_SPEC.md` is the current source of truth for the planned architecture.

## Key Current Facts

- This is effectively greenfield from a public runtime-API perspective.
  - There are no known external consumers relying on the current advanced `Runtime` hook surface.
- The current branch includes substantial Ruby adapter machinery in:
  - `crates/ruby-ext`
  - `harness/ruby-liquid`
- Current code includes compatibility mechanisms for:
  - Ruby-defined filters/tags/blocks
  - Ruby exception rendering behavior
  - Ruby-visible `ResourceLimits`
  - live scope visibility during callbacks from Ruby drops/procs

## Decisions Already Made

These are already agreed and should be treated as plan constraints unless you find a strong reason they are wrong:

- Use an internal sealed render-policy API, not a generic hook bag.
- Production render path uses `ProdPolicy`.
- Ruby conformance path uses `RubyConformancePolicy`.
- Public production engine APIs must not become generic over policy.
- `ruby-ext` is the only supported consumer of the conformance-only path.
- Conformance access should be via a `#[doc(hidden)] pub` feature-gated `conformance` module.
- `ruby-ext` owns the conformance environment adapter for Ruby-defined filters/tags/blocks.
- Conformance parsing should use a dedicated tokenizer / parse prepass in `ruby-ext`, not long-term placeholder string rewriting.
- Production should have abuse protection.
- Conformance should keep exact Ruby `ResourceLimits`.
- Production public API should expose explicit safety limits instead of the Ruby `ResourceLimits` API shape.
- The initial production safety-limit API is:
  - `RenderOptions { max_output_bytes, max_render_ops, max_assign_bytes }`
  - `Template::render_with_options(...)`
  - `Template::render_to_with_options(...)`
- Production limits should be cumulative across the whole render tree and should not reset at partial boundaries.
- Ruby reset semantics remain conformance-only.
- Keep one workspace for now; enforce isolation with `default-members` and separate CI lanes rather than splitting workspaces immediately.

## What Review Is Wanted

Please review the plan for:

- architectural ambiguity
- migration risk
- hidden coupling between production and Ruby conformance code
- places where the plan leaves important implementation decisions unspecified
- places where the design would accidentally preserve Ruby-specific semantics in production
- public API risks or awkwardness in the proposed production safety-limit API

## What Review Is Not Needed

- Do not spend effort re-arguing whether we should keep Ruby in production. That decision is already made: Ruby is conformance-only.
- Do not review the branch as a code-quality pass unless it directly affects the architectural plan.
- Do not propose implementation work unless it is necessary to explain a flaw in the plan.
