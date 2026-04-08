# Contract: Conformance Harness Bridge

## Scope

Unstable, feature-gated contract between `liquid-core`/`liquid` and `crates/ruby-ext`. This surface exists only to run the upstream Shopify Liquid suite and is not part of the supported production API.

## Build Contract

- `conformance-harness` is a non-default Cargo feature rooted in `liquid-core` and propagated through `liquid-lib` and `liquid`.
- The hidden conformance module is compiled and re-exported only when `conformance-harness` is enabled.
- Default builds must not compile `LiveScopeSession`, Ruby resource-limit adapters, fallback filter dispatchers, or other conformance-only state.
- `crates/ruby-ext` is the only supported consumer of this contract.

## Runtime Contract

- Conformance entrypoints install `RubyConformancePolicy` instead of the production policy.
- Ruby-defined filters, tags, and blocks are registered through existing parser extension points and remain available to the harness path.
- Late-registered Ruby filters and render-time overrides are resolved through a register-carried fallback dispatcher before production-style unknown-filter handling is applied.
- `SandboxedStackFrame` must propagate the active policy handle, fallback filter dispatcher, and live-scope/adapter state into isolated render scopes.
- Ruby-specific error normalization remains in `crates/ruby-ext`; production engine code provides raw `Error` values and policy callbacks only.

## Validation Contract

- Compatibility claims are scoped to Shopify Liquid commit `a9c85622ddd784078c2eed34b19a351fe57362cf`.
- The validation command is `make harness-test`.
- Passing the pinned suite is the compatibility claim; re-pinning requires updating the recorded baseline and rerunning the full conformance lane.

## Stability Contract

- This contract is intentionally hidden and semver-unstable.
- Changes are acceptable when required to preserve the pinned Shopify Liquid behavior or to keep Ruby-specific machinery out of the production surface.
