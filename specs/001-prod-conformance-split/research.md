# Phase 0 Research: Pure-Rust Production Engine with Ruby-Only Conformance Harness

All technical-context unknowns needed for planning are resolved. The decisions below translate the active feature spec and local design history into an implementation-ready baseline.

## Decision 1: Keep one workspace and enforce the production boundary with `default-members` plus a feature gate

- **Decision**: Keep the existing Cargo workspace, add production-only `default-members`, preserve `default = ["stdlib"]`, and gate conformance-only code behind a `conformance-harness` feature rooted in `liquid-core` and propagated through `liquid-lib` and `liquid`.
- **Rationale**: This gives Ruby-free default `cargo check` and `cargo test` while preserving a single repo, single dependency graph, direct `#[cfg(feature = "conformance-harness")]` gating where registers, stack frames, and stdlib tags actually live, and the constitution-required ability to build a minimal parser with `--no-default-features`.
- **Alternatives considered**: Split the repository into two workspaces; use ad-hoc `RUSTFLAGS` cfg toggles; keep conformance code always compiled and rely on runtime flags.

## Decision 2: Make `Template::render_to` the executor and move policy out of `Runtime`

- **Decision**: Treat `crates/core/src/runtime/template.rs` as the executor, retrieve the active policy from registers, and remove scoring, limit, and render-error hooks from the public `Runtime` trait.
- **Rationale**: The executor already owns the render loop and is the right place to wrap each `Renderable::render_to` call. This keeps `Runtime` data-only, matches the active spec, and avoids a generic runtime hook surface that would keep Ruby-shaped behavior in production.
- **Alternatives considered**: Keep the current hook-heavy `Runtime`; pass a policy parameter through every renderable; create a separate executor object disconnected from `Template::render_to`.

## Decision 3: Expose explicit production render controls through `RenderOptions`, `ErrorMode`, and `RenderOutput`

- **Decision**: Add public `RenderOptions`, `ErrorMode`, and `RenderOutput` types, and make `Template::render`, `Template::render_to`, and `RenderOptions::default()` use Ruby-compatible lenient defaults for undefined variables, undefined filters, and render-error handling.
- **Rationale**: The feature spec now explicitly requires default lenient behavior. A dedicated options/result API makes abuse protection and collected-error reporting first-class without leaking conformance-only implementation details into the stable public API.
- **Alternatives considered**: Keep today’s strict defaults; expose only strict mode in v1; model lenient behavior through booleans and optional callbacks instead of dedicated types.

## Decision 4: Keep late filter overrides inside the conformance bridge rather than the stable production API

- **Decision**: Support late-registered filter overrides through a feature-gated fallback dispatcher stored in registers for conformance builds. Production continues to rely on Rust-native parse-time registration plus `strict_filters` behavior for unresolved filters.
- **Rationale**: The active feature spec requires late registration for Ruby conformance, not as a stable production API promise. Keeping the dispatcher in the conformance path avoids reintroducing a generic filter-interposition surface in production.
- **Alternatives considered**: Make the fallback dispatcher available in all builds; keep `Runtime::evaluate_filter`; patch the parsed AST at render time.

## Decision 5: Represent per-render mutable state as a single-threaded shared handle installed in registers

- **Decision**: Represent production render state as per-render policy state (`render_ops`, `assign_bytes`, `depth`, collected errors) shared through registers with `Rc` plus interior mutability, and propagate that shared handle through nested scopes.
- **Rationale**: Each top-level render remains isolated, while nested partials and `{% render %}` calls share the same counters and collected errors. This preserves `Template` sharing across threads without introducing `Arc<Mutex<_>>` overhead into the render hot path.
- **Alternatives considered**: Global/shared counters across renders; `Arc<Mutex<_>>` policy state; separate counters per partial boundary.

## Decision 6: Treat benchmark execution as a merge requirement, not a feature success criterion

- **Decision**: Do not encode benchmark stability as a formal success criterion in the feature spec, but still require benchmark maintenance and `cargo bench` execution before merge under the constitution.
- **Rationale**: The spec needed measurable acceptance criteria, and “measurement noise” was too vague. The constitution still requires benchmark discipline for performance-sensitive changes, so benchmark review remains part of implementation and merge validation.
- **Alternatives considered**: Keep a vague benchmark requirement in the spec; choose an arbitrary percentage threshold without user confirmation; skip benchmark work entirely.

## Decision 7: Use the pinned upstream commit plus the currently executable harness toolchain as the operational compatibility baseline

- **Decision**: Lock compatibility claims to Shopify Liquid commit `a9c85622ddd784078c2eed34b19a351fe57362cf` and use the current harness script/README Ruby toolchain expectation (`RBENV_VERSION=3.4.1`) for implementation and validation.
- **Rationale**: The commit pin is now explicit in the spec and must drive all compatibility claims. The harness script and README both point to Ruby 3.4.1, which makes them the most reliable operational baseline available in the repo today.
- **Alternatives considered**: Leave the upstream pin abstract; plan around `harness/baseline.yml`’s stale Ruby 3.2.2 entry without reconciling it to the executable harness script; defer environment normalization until after implementation.
