# Feature Specification: Pure-Rust Production Engine with Ruby-Only Conformance Harness

**Feature Branch**: `001-prod-conformance-split`
**Created**: 2026-04-04
**Status**: Draft
**Input**: User description: "PRODUCTION_CONFORMANCE_SPLIT_SPEC.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Production Rendering Without Ruby (Priority: P1)

A library consumer embeds liquid-rust in a Rust application to render
Liquid templates. They build their project with the default feature set.
The build completes without requiring Ruby, `magnus`, or any
conformance-only code. They render templates with custom filters, tags,
and blocks registered through the Rust API and receive correct output
matching Shopify Liquid semantics for the pinned upstream Shopify
Liquid commit `a9c85622ddd784078c2eed34b19a351fe57362cf`.

**Why this priority**: This is the core value proposition — a clean,
pure-Rust production engine. Every other story depends on this boundary
being enforced.

**Independent Test**: Build with default features, run root
`cargo check` and `cargo test` using Cargo `default-members` without
Ruby installed, and run a stdlib-disabled regression proving
`ParserBuilder` still supports a minimal parser with zero built-in
filters. Verify all production tests pass and no conformance code
compiles.

**Acceptance Scenarios**:

1. **Given** a Rust project depending on `liquid` with default features,
   **When** the project is built, **Then** the build succeeds without
   Ruby installed and no conformance-only modules are compiled.
2. **Given** a parsed template and globals, **When** `template.render(&globals)`
   is called, **Then** the output matches the expected Shopify Liquid
   behavior for the pinned upstream Shopify Liquid commit
   `a9c85622ddd784078c2eed34b19a351fe57362cf`.
3. **Given** a template with custom Rust filters/tags/blocks registered
   via `ParserBuilder`, **When** rendered, **Then** custom extensions
   execute correctly and the engine uses only Rust-native dispatch.
4. **Given** a consumer depends on `liquid` with
   `default-features = false` and registers custom Rust extensions via
   `ParserBuilder`, **When** a template uses only those custom
   extensions, **Then** parsing and rendering succeed with zero
   built-in stdlib filters present.

---

### User Story 2 - Configurable Render Limits and Error Handling (Priority: P2)

A library consumer renders user-supplied templates in a multi-tenant
environment. They configure render limits (max output bytes, max render
ops, max assign bytes, max nesting depth) and choose between strict
error mode (abort on first error) and lenient mode (inline errors,
continue rendering). Limits are cumulative across the entire render tree
including nested partials.

**Why this priority**: Abuse protection is essential for any production
deployment that accepts untrusted templates. Lenient error mode matches
Ruby Liquid's default behavior and is required for conformance parity.

**Independent Test**: Render templates with various `RenderOptions`
configurations and verify limits are enforced and errors are handled
per the selected mode.

**Acceptance Scenarios**:

1. **Given** `RenderOptions` with `max_output_bytes: Some(1024)`,
   **When** a template produces more than 1024 bytes, **Then** rendering
   aborts mid-write with an "output limit exceeded" error.
2. **Given** `RenderOptions` with `max_render_ops: Some(100)`,
   **When** a template evaluates more than 100 AST nodes (including
   for-loop iterations), **Then** rendering aborts with an error.
3. **Given** `ErrorMode::Strict`, **When** a render error occurs,
   **Then** `render_with_options` returns `Err` immediately.
4. **Given** `ErrorMode::Lenient(formatter)`, **When** render errors
   occur, **Then** formatted error strings appear inline in the output
   AND errors are collected in `RenderOutput.errors`.
5. **Given** `strict_variables: false` (the default), **When** a
   template references an undefined variable, **Then** it resolves
   to nil with no error, matching Ruby Liquid defaults.
6. **Given** `strict_filters: false` (the default), **When** a
   template uses an undefined filter, **Then** the input passes
   through unchanged with no error.
7. **Given** `RenderOptions::default()` or `template.render(&globals)`,
   **When** a render error occurs, **Then** rendering uses lenient
   error handling by default and returns inline error output instead
   of aborting on the first error.

---

### User Story 3 - Full Shopify Conformance Via Ruby Harness (Priority: P3)

A project maintainer verifies that the engine passes 100% of the pinned
upstream Shopify Liquid Ruby test suite. They build with the
`conformance-harness` feature enabled, which activates conformance-only
code paths (Ruby policy, fallback filter registry, live-scope session,
conformance entrypoints). The Ruby harness runs the upstream suite
through `ruby-ext` and all tests pass.

**Why this priority**: Conformance is the project's compatibility claim.
However, the conformance harness is a development/CI tool, not part of
the production runtime, so it ranks below production functionality.

**Independent Test**: Build with `conformance-harness` feature, run
`make harness-test`, verify 100% pass
rate against Shopify Liquid commit
`a9c85622ddd784078c2eed34b19a351fe57362cf`.

**Acceptance Scenarios**:

1. **Given** a build with `conformance-harness` feature enabled,
   **When** `ruby-ext` constructs a conformance environment with
   Ruby-defined filters, tags, and blocks, **Then** parsing and
   rendering use the conformance code paths (Ruby policy, fallback
   filter registry, live-scope notifications).
2. **Given** the full pinned Shopify Liquid upstream test suite,
   **When** run through the conformance harness, **Then** all tests
   pass with zero failures.
3. **Given** a production build without `conformance-harness`,
   **When** inspecting compiled output, **Then** no conformance-only
   code (live-scope session, Ruby resource limits, fallback filter
   registry) is present.

---

### User Story 4 - Concurrent Template Rendering (Priority: P4)

A library consumer shares a parsed `Template` across multiple threads
and renders it concurrently with different globals. Each render operates
in isolation — its own runtime, registers, policy state, and counters.
No data races or shared mutable state occur between concurrent renders.

**Why this priority**: Thread safety is essential for server-side use
but is a property of the architecture, not a user-facing feature.

**Independent Test**: Spawn multiple threads sharing a single
`Template`, render with different globals concurrently, verify correct
isolated output.

**Acceptance Scenarios**:

1. **Given** a parsed `Template` shared across threads, **When**
   multiple threads call `render` or `render_with_options`
   concurrently, **Then** each render produces correct output with
   no data races.
2. **Given** concurrent renders with different `RenderOptions`,
   **When** one render hits a limit or error, **Then** other
   concurrent renders are unaffected.

---

### Edge Cases

- What happens when a `for` loop iterates over millions of items with
  `max_render_ops` set? The ops counter accumulates per-iteration body
  nodes and aborts when the limit is reached.
- What happens when `{% render %}` creates an isolated scope? The
  `SandboxedStackFrame` propagates the active policy and adapter state
  from parent registers, so limits and error handling remain consistent.
- What happens when a template uses an undefined filter with
  `strict_filters: false` and `ErrorMode::Strict`? The filter returns
  the input unchanged — no error is produced because the filter
  resolution itself is not an error in lenient filter mode.
- What happens when `max_depth` is exceeded inside a deeply nested
  `{% include %}` chain? A "nesting too deep" error is returned during
  stack frame construction.
- What happens when a conformance build encounters a filter registered
  at render time (not parse time)? The `FallbackFilterRegistry`
  dispatches it at render time, matching Ruby's strainer behavior.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Production builds MUST compile and pass all tests without
  Ruby, `magnus`, or any conformance-only code.
- **FR-002**: The `Runtime` trait MUST expose only data-access methods
  (partials, variable get/set, registers). Scoring, error handling,
  resource limits, and filter dispatch MUST NOT be on `Runtime`.
- **FR-003**: The engine MUST provide a sealed render-policy abstraction
  with two implementations: one for production and one for conformance.
- **FR-004**: Production render MUST support configurable limits:
  `max_output_bytes`, `max_render_ops`, `max_assign_bytes`, and
  `max_depth`, each independently optional (unlimited when unset).
- **FR-005**: Production render MUST support both strict and lenient
  error modes, with lenient mode writing formatted errors inline and
  collecting them for programmatic access.
- **FR-006**: Production limits MUST be cumulative across the entire
  render tree and MUST NOT reset at partial boundaries.
- **FR-007**: `max_output_bytes` MUST be enforced mid-write (not just
  at element boundaries), providing a hard limit on output size.
- **FR-008**: Default `render()`, `render_to()`, and
  `RenderOptions::default()` MUST use Ruby-compatible lenient defaults:
  `strict_variables: false`, `strict_filters: false`, and lenient
  render-error handling.
- **FR-009**: `strict_variables: false` MUST be enforced as a
  runtime-level lenient lookup mode across the entire render tree,
  not only as a root-globals wrapper.
- **FR-010**: Conformance-only code MUST be gated behind the
  `conformance-harness` feature and MUST NOT compile in production.
- **FR-011**: The conformance path MUST support late-registered filters
  (registered between parse and render) via a fallback filter registry.
- **FR-012**: The conformance path MUST support Ruby-defined custom tags
  and blocks registered into the parser via existing extension points.
- **FR-013**: Live-scope session state MUST exist only in conformance
  builds and compile away entirely in production.
- **FR-014**: `SandboxedStackFrame` (used by `{% render %}`) MUST
  propagate the active policy, fallback filter registry, and adapter
  state from parent to child registers.
- **FR-015**: `Template` MUST be safe to share across threads. Each
  render MUST create its own isolated runtime and policy state.
- **FR-016**: The Shopify Liquid test suite for commit
  `a9c85622ddd784078c2eed34b19a351fe57362cf` MUST pass 100% through
  the conformance harness.
- **FR-017**: Error message normalization for Ruby compatibility MUST
  remain in the conformance adapter, not in the production engine.
- **FR-018**: The refactor MUST be delivered as a single change. The
  conformance harness MUST remain green at the end.
- **FR-019**: The public `ParserBuilder` extension surface MUST remain
  usable when `liquid` is built with `default-features = false`,
  allowing consumers to construct a minimal parser with zero built-in
  stdlib filters.

### Key Entities

- **RenderOptions**: Configuration for a single render: limits
  (output bytes, render ops, assign bytes, depth), strictness modes
  (variables, filters), and error handling mode.
- **ErrorMode**: Controls error behavior — strict (abort) or lenient
  (inline + collect), with a configurable formatter for lenient mode.
- **RenderOutput**: Result of a configurable render — contains the
  rendered output string and any collected errors.
- **RenderPolicy**: Internal sealed abstraction with production and
  conformance implementations. Handles render-op counting, error
  handling, assign tracking, and scope depth.
- **FallbackFilterRegistry**: Conformance-only registry for filters
  registered at render time. Dispatches by name with pre-evaluated
  arguments.
- **Runtime**: Data-only trait for variable access, partials, and
  registers. No policy methods.
- **CountingWriter**: Writer wrapper that enforces `max_output_bytes`
  mid-write.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Production root `cargo check` and `cargo test` pass with
  zero failures using Cargo `default-members` and without Ruby
  installed.
- **SC-002**: No conformance-only code compiles in a default
  (production) build, verifiable by feature-off compilation tests.
- **SC-003**: 100% of the Shopify Liquid test suite for commit
  `a9c85622ddd784078c2eed34b19a351fe57362cf` passes through the
  conformance harness.
- **SC-004**: Concurrent rendering of a shared template from multiple
  threads produces correct, isolated results with no data races.
- **SC-005**: All four production limits (`max_output_bytes`,
  `max_render_ops`, `max_assign_bytes`, `max_depth`) correctly abort
  rendering when exceeded, verified by dedicated tests.
- **SC-006**: Lenient error mode correctly inlines formatted errors in
  output AND collects them in `RenderOutput.errors`, including when
  using `RenderOptions::default()`.
- **SC-007**: `strict_variables: false` resolves undefined variables to
  nil across the full render tree, including nested and isolated scopes.
- **SC-008**: A stdlib-disabled regression proves `ParserBuilder`
  remains usable for a minimal parser with zero built-in filters and
  custom Rust extensions.

## Clarifications

### Session 2026-04-04

- Q: What release version should this breaking refactor target? → A: 1.0.0 (constitution-compliant major bump for breaking public API changes)
- Q: Should parser/render fuzzing be a formal requirement for this refactor? → A: No; fuzzing is a separate quality bar, not a blocker for this refactor
- Q: Which default error mode should the public production API use when callers do not provide `RenderOptions`? → A: `render()`, `render_to()`, and `RenderOptions::default()` use lenient error handling by default
- Q: What benchmark regression threshold should the benchmark success criterion enforce for this refactor? → A: Drop the benchmark-specific success criterion; benchmark stability is not a blocking success criterion for this refactor
- Q: Should this spec lock the conformance target to the exact upstream Shopify Liquid commit already recorded in `harness/baseline.yml`? → A: Pin the spec to commit `a9c85622ddd784078c2eed34b19a351fe57362cf`
- Q: How should the spec define the production verification command for Story 1 and `SC-001`? → A: Use root `cargo test` / `cargo check` with Cargo `default-members` only

## Assumptions

- No external consumers depend on the current `Runtime` hook surface or
  `render_to_runtime` API, so breaking changes are acceptable.
- Production only needs Rust-native extensibility; Ruby-style dynamic
  host callbacks are not required in production.
- A single workspace with `default-members` and feature gating provides
  sufficient prod/conformance isolation; a second workspace is deferred.
- The upstream Shopify Liquid test suite at commit
  `a9c85622ddd784078c2eed34b19a351fe57362cf` is the authoritative
  source of truth for behavioral conformance.
- Changing default `render()` behavior to `strict_variables: false` and
  `strict_filters: false` (matching Ruby) is an intentional, acceptable
  breaking change.
- The refactor is delivered as one large PR; incremental delivery is not
  required for this change.
- This refactor ships as version 1.0.0. The constitution requires a
  major bump for breaking public API changes.
- Parser/render fuzzing is a separate quality bar tracked outside this
  spec. It is not a blocking requirement for this refactor.
