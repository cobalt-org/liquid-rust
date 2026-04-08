# Spec: Pure-Rust Production Engine with Ruby-Only Full Conformance Harness

## Summary

- Treat the project as greenfield for runtime API purposes: no external consumers depend on the current `Runtime` hook surface, so the cleanup can be intentionally breaking.
- Production becomes a pure Rust Liquid engine with Rust-native extensibility only.
- Ruby remains only as a full external conformance harness.
- Use an internal sealed render-policy API, not a generic hook bag.
- Production should have abuse protection.
- Conformance should keep Ruby `ResourceLimits` exactly.
- Production public API should expose explicit safety limits; Ruby-style counters remain internal implementation detail at most.
- Keep one workspace and enforce the prod/test boundary with `default-members`, conformance-only features, and separate CI lanes.

## North Star

- The goal is a pure-Rust production engine that matches the pinned Shopify Ruby Liquid revision closely enough to pass 100% of the exact upstream Ruby suite through the conformance harness.
- The Ruby harness is an external referee, not part of the long-term production runtime architecture.
- The point of the split is to get Ruby-compatible behavior with clean Rust internals, not to preserve Ruby-shaped implementation structure inside production code.
- The defining compatibility claim for this repo is intentionally strong: this fork aims to be a full Shopify Liquid compatibility port, and it validates that claim by running Shopify Liquid's own Ruby tests.
- Full upstream-suite pass is necessary for the compatibility claim, but it is not proof of zero bugs. Production quality also depends on Rust-native verification aimed at engine-owned behavior and failure modes not fully covered by the Ruby suite.

## Compatibility Contract

- “100% compatibility” means the pinned upstream Shopify Liquid Ruby suite passes completely against the conformance harness for the selected upstream commit.
- Compatibility claims are scoped to that pinned upstream revision. Green at one pin does not automatically imply compatibility with future upstream revisions.
- The upstream suite remains the behavioral source of truth for shared Liquid semantics and Ruby-visible conformance behavior. This repo intentionally uses Shopify Liquid's own Ruby tests as the compatibility oracle.
- Production-only features may be introduced during this refactor, but they must be explicitly configured or otherwise proven not to change the default Ruby-compatible behavior claimed by this spec.
- Passing the upstream suite is the primary compatibility bar; it does not imply identical performance, memory profile, or internal implementation structure.

## Clean Rust Implementation Contract

- Production engine state must not store Ruby objects or Ruby-runtime handles.
- Production code must not depend on `ruby-ext`, `magnus`, or conformance-only modules/features.
- Production must not expose a generic runtime callback/interposition API just to support Ruby harness behavior.
- Conformance-only behavior must stay behind the `conformance-harness` feature and the hidden conformance entrypoints.
- The production design is allowed to diverge internally from Ruby Liquid as long as the externally observable behavior remains compatible with the pinned suite and the production API guarantees in this spec.

## Version Drift Policy

- The project tracks compatibility against an explicit pinned upstream commit.
- Re-pinning upstream is a deliberate re-baselining event, not a background maintenance detail.
- When the pin changes, the new pin must be recorded and the full conformance lane must be rerun before any new compatibility claim is made.
- A green result at the current pin is the compatibility claim. It is not a standing claim about upstream `main` or future gem releases.

## Key Changes

### 1. Enforce a real prod/test boundary

- Keep a single Cargo workspace, but add `default-members` for production crates only so root `cargo check` / `cargo test` do not build `crates/ruby-ext`.
- Production CI uses only default-members and must not require Ruby.
- Conformance CI explicitly builds `liquid_ext` and runs the full Shopify harness with `make harness-test`.
- Do not add a second workspace in phase 1. Revisit only if one-workspace isolation still proves insufficient after `default-members` and CI separation are in place.
- `conformance-harness` is a feature on `liquid-core`, propagated through `liquid-lib` and `liquid` via feature unification. This lets `Registers`, stack frames, and stdlib tags all use `#[cfg(feature = "conformance-harness")]` to gate conformance-only code paths. Production builds never compile conformance code.

### 2. Shrink `Runtime` to a data-only trait; move policy hooks to the executor

- `Runtime` keeps only data-access and engine-state methods:
  - `partials`, `name`, `roots`
  - `try_get`, `get`, `set_global`, `set_global_range`, `set_global_alias`
  - `set_index`, `get_index`, `get_global_range_bounds`
  - `registers`
- `set_global_range` and `get_global_range_bounds` are part of the canonical runtime surface because assigned ranges must stay lazy instead of being eagerly materialized into arrays during normal renders.
- Remove from `Runtime`:
  - `evaluate_filter` (override hook; Rust-only dispatch becomes a non-overridable function in `filter_chain.rs`)
  - `handle_render_error`
  - `increment_render_score`, `increment_assign_score`
  - `check_resource_limits`, `reset_resource_limits`
- The public `Template` API remains `Template::render`, `Template::render_to`, `Template::render_with_options`, and `Template::render_to_with_options`; conformance-specific render hooks remain internal and are not part of the public `Template` surface.
- Production extensibility remains Rust-native only: filters, tags, blocks, and partials registered in Rust. Any special conformance runtime entrypoints are internal implementation details, not public extension points.
- `Renderable::render_to(&self, writer, runtime)` keeps its current `&dyn Runtime` signature. Tags and blocks use `Runtime` only for data operations — this matches Ruby Liquid's design where tags call `context.evaluate` / `context[]` / `context.invoke` but never directly call resource-limit or error-handling methods.

### 3. Introduce an internal sealed render-policy API and executor

- Add an internal, sealed render-policy abstraction inside core/lib.
- This policy API is not public extension surface and is not part of the Rust compatibility promise.
- The engine defines two policy implementations:
  - `ProdPolicy` for all production renders (strict and lenient modes)
  - `RubyConformancePolicy` for the Ruby adapter path only
- Public production render entrypoints always use `ProdPolicy`.
- Conformance-only parse/render entrypoints use `RubyConformancePolicy` and are hidden behind a non-default `conformance-harness` feature.
- Keep public `Template`, parser types, and production-facing APIs non-generic.

#### `ProdPolicy` construction and state

`ProdPolicy` is split into immutable config copied from `RenderOptions` plus a shared per-render state handle. The handle is created fresh for each top-level render and shared across the entire nested render tree. “Immutable” here means “fixed for the lifetime of one top-level render.” Limits and error mode are render inputs; only counters and collected errors mutate:

```rust
struct ProdPolicy {
    // Immutable config (copied from RenderOptions)
    max_render_ops: Option<usize>,
    max_assign_bytes: Option<usize>,
    max_depth: Option<usize>,
    error_mode: ErrorMode,

    // Shared per-render state
    state: Rc<ProdPolicyState>,
}

struct ProdPolicyState {
    render_ops: Cell<usize>,
    assign_bytes: Cell<usize>,
    depth: Cell<usize>,
    errors: RefCell<Vec<Error>>,
}
```

- The state handle is single-threaded and per-render. Use `Rc` + `Cell`/`RefCell`, not `Arc<Mutex<_>>`.
- Each top-level render creates its own state handle. Concurrent renders of the same template never share policy state.
- Nested partials and nested `render` calls share the same state handle, so production counters and collected lenient errors remain cumulative across the whole render tree.
- The depth counter is part of the shared policy state — not a separate register.
- `max_output_bytes` is not in the policy — it's handled by `CountingWriter` directly.
- `errors` collects handled errors in lenient mode. After rendering, the entrypoint extracts and drains them from the shared state.

The public entrypoint constructs and installs the policy:

```rust
pub fn render_with_options(&self, globals, options) -> Result<RenderOutput> {
    let policy = ProdPolicy::from_options(options);
    let runtime = RuntimeBuilder::new().set_globals(globals).build();
    // install policy in registers
    // install CountingWriter with max_output_bytes from options
    // ... render ...
    // extract policy from registers, drain errors into RenderOutput
}
```

#### `RenderPolicy` trait surface

The sealed trait has exactly 5 methods, one per call-site concept:

```rust
trait RenderPolicy {
    /// Executor calls before each Renderable::render_to.
    /// Increments ops counter, checks ops and assign-bytes limits.
    fn on_render_op(&mut self) -> Result<()>;

    /// Executor calls when render_to returns Err.
    /// Strict: returns Err (abort). Lenient: pushes to collected errors,
    /// returns Ok(Some(formatted_string)) for inline output.
    fn on_render_error(&mut self, error: Error) -> Result<Option<String>>;

    /// assign_tag / capture_block call with byte count.
    fn on_assign(&mut self, bytes: usize) -> Result<()>;

    /// StackFrame / SandboxedStackFrame call on construction.
    /// partial=true for include/render (triggers conformance reset), false for for-loops.
    /// Increments depth, checks max_depth.
    fn on_scope_push(&mut self, partial: bool) -> Result<()>;

    /// StackFrame / SandboxedStackFrame call on drop. Decrements depth. Infallible.
    fn on_scope_pop(&mut self);
}
```

- `on_render_op` combines scoring and limit-checking in one call — no separate `increment_render_score` + `check_resource_limits`.
- `on_scope_push(partial)` folds depth tracking and partial-entry into one call — the spec says they share a call site.
- `max_output_bytes` is enforced by `CountingWriter`, not by the policy — no method for it.

#### Executor owns all policy calls

- The render loop (currently `crates/core/src/runtime/template.rs` `Template::render_to`) becomes the executor. It retrieves the active `RenderPolicy` from registers and uses it to drive the element loop.
- The executor — not individual renderables — calls `on_render_op` and `on_render_error` through the policy.
- `Renderable::render_to` continues to receive `&dyn Runtime` (data-only). The executor wraps each `render_to` call with the policy hooks: call `on_render_op`, call `render_to`, handle errors via `on_render_error`.
- Stack frame types (`StackFrame`, `SandboxedStackFrame`, `GlobalFrame`) no longer forward scoring/error/limit methods — those methods no longer exist on `Runtime`.

#### Policy is stored in registers and accessible at every recursion depth

- The active `RenderPolicy` is stored in registers as a shared per-render handle accessible through `runtime.registers()`.
- `Template::render_to` retrieves the policy from registers to drive its element loop. Since container tags (`for`, `if`, `case`, etc.) all render their bodies by calling `Template::render_to` on child `Template` instances, the policy is naturally available at every recursion depth without changing the `Renderable` trait.
- The public render entrypoints (`Template::render`, `Template::render_with_options`) install the appropriate policy in registers before rendering and remove it after.
- This keeps `Runtime` data-only at the trait level while making the policy available to the executor at every recursion depth.

#### `SandboxedStackFrame` must propagate policy and adapter state

- `SandboxedStackFrame` (used by the `render` tag) creates its own isolated `Registers` instance (`stack.rs:342`), unlike `StackFrame`/`GlobalFrame`/`IndexFrame` which forward `registers()` to their parent.
- Currently `SandboxedStackFrame::new` only copies `live_scope_session` from the parent registers (`stack.rs:343`). After the refactor, it must also propagate the active policy register, the fallback filter registry, and any conformance adapter state from parent to child registers during construction.
- Interrupt registers and other per-scope state remain isolated — only shared handles such as the policy and adapter state are propagated. The policy/state handle must not be copied by value.
- Without this propagation, rendered partials (via `{% render %}`) would silently fall back to a default policy, breaking error handling, resource-limit resets, filter dispatch, and live-scope notifications inside rendered partials.

#### Assign scoring is reported from inside renderables (narrow exception)

- Assign sizes are known only inside `assign_tag.rs` and `capture_block.rs` — the executor at the `render_to` boundary cannot know how many bytes were assigned.
- These two tags access the active policy through registers and call `on_assign`. This is one of three narrow exceptions where code inside a `render_to` call interacts with the policy through registers.
- This is a narrow exception to "tags use Runtime for data only": these two tags report a measurement but make no policy decisions.
- The executor checks the cumulative assign limit at element boundaries after each `render_to` returns.

#### Depth tracking is reported from stack frame construction (narrow exception)

- Depth increments happen when tags create `StackFrame` or `SandboxedStackFrame` — this occurs inside a tag's `render_to`, not at the executor boundary. The executor can't see scope creation.
- `StackFrame::new` and `SandboxedStackFrame::new` increment the depth counter in registers during construction, check against `max_depth`, and decrement when the frame is dropped or scope exits. This is analogous to Ruby's `check_overflow` being called inside `Context#push` and `Context#new_isolated_subcontext`.
- Only `for` (creates `StackFrame` per iteration), `include` (creates `StackFrame`), and `render` (creates `SandboxedStackFrame`) push scopes. `if` and `case` do not create new scopes — they render their branches with the same runtime.
- This is the second narrow exception to "tags use Runtime for data only": stack frame construction reports depth, but makes no policy decisions beyond checking the limit.

#### Filter dispatch stays in `filter_chain.rs`, not in the executor

- Filter dispatch is not an executor-boundary operation. It happens inside `filter_chain.rs` during expression evaluation within a renderable's `render_to`.
- The current two-path design in `filter_chain.rs` has `ParsedFilter::Compiled` (filter found in registry at parse time, called directly) and `ParsedFilter::Deferred` (filter not in registry, dispatched at render time via `runtime.evaluate_filter`).
- After the refactor, `runtime.evaluate_filter` is removed. Filter dispatch becomes:
  - Production path:
    - `ParsedFilter::Compiled`: unchanged — filter was in the `PluginRegistry` at parse time, `Filter::evaluate()` is called directly at render time.
    - `ParsedFilter::Deferred`: no runtime dispatcher exists in production. After evaluating arguments, `filter_chain.rs` either returns the input unchanged (`strict_filters: false`) or raises an unknown-filter error (`strict_filters: true`).
  - Conformance path:
    - Before executing either parsed filter variant, `filter_chain.rs` first consults the active conformance filter dispatcher for a render-time override by filter name.
    - If a render-time override exists, it is used for the call even when the filter was known at parse time.
    - If no override exists, `ParsedFilter::Compiled` uses the direct compiled path and `ParsedFilter::Deferred` uses the conformance fallback registry path.
- This preserves Shopify Ruby behavior where render-time `filters:` overrides can replace globally known filters that were already available at parse time.

#### Partial-entry hook replaces direct `reset_resource_limits` calls (narrow exception)

- The direct `reset_resource_limits()` calls in `render_tag.rs` and `include_tag.rs` are replaced with `on_scope_push(true)` — the same policy method used for depth tracking, with `partial=true` signaling a partial boundary.
- `ProdPolicy::on_scope_push` ignores the `partial` flag (no reset, cumulative-only). `RubyConformancePolicy::on_scope_push` resets per-template counters when `partial=true`, per Ruby semantics.
- This is the third narrow exception to "tags use Runtime for data only": include/render tags invoke a policy hook but make no policy decisions.
- The partial-entry behavior and depth tracking share the same call (`on_scope_push`), so these two exceptions are a single policy call in practice.
- The executor cannot trigger this — it iterates `Box<dyn Renderable>` and can't distinguish partial renders from other elements.

### 3a. Conformance fallback filter registry for late-registered filters

- Ruby Liquid supports adding filters between parse and render (via `template.render(assigns, filters: [...])` or `context.add_filters`). Filters are never needed at parse time in Ruby — they're always resolved at render time via the strainer.
- To support this pattern in conformance builds, a shared per-render `FallbackFilterRegistry` is stored in registers behind `conformance-harness`.
- When `filter_chain.rs` encounters a `ParsedFilter::Deferred` filter:
  1. It evaluates the positional and keyword argument expressions against the runtime.
  2. It looks up the filter by name in the `FallbackFilterRegistry`.
  3. It calls the filter's dispatch function directly with the evaluated arguments — no `ParseFilter::parse()` step.

#### `FallbackFilterRegistry` concrete type

```rust
pub trait FilterDispatcher {
    fn invoke(
        &self,
        name: &str,
        input: &dyn ValueView,
        positional: &[Value],
        keyword: &[(String, Value)],
        runtime: &dyn Runtime,
    ) -> Result<Value>;
}

#[derive(Default)]
pub struct FallbackFilterRegistry {
    dispatcher: Option<Rc<dyn FilterDispatcher>>,
    strict_filters: bool,  // default false
}
```

- `Default` is `None` dispatcher + `strict_filters: false`. When no filter resolution path succeeds: `strict_filters=true` → "unknown filter" error, `strict_filters=false` → return input value unchanged (matching Ruby's lenient behavior).
- The registry is a shared per-render handle. Use `Rc`, not `Arc`, because a single render is single-threaded but nested `render` calls must see the same dispatcher.
- `FilterDispatcher` intentionally does **not** require `Send + Sync`. It is conformance-only, per-render, and single-threaded. The Ruby-backed dispatcher that delegates to the strainer is allowed to be single-threaded too.
- The dispatcher handles name-based lookup internally. Ruby's dispatcher routes all names to the strainer. The registry doesn't need to know how lookup is implemented.
- This avoids the per-render heap allocation of a fresh boxed filter object for deferred filters. `filter_chain.rs` evaluates arguments itself, then hands evaluated values to the dispatcher.

#### Usage

- In conformance, `ruby-ext` installs a `FallbackFilterRegistry` with a dispatcher that delegates to the Ruby strainer.
- `SandboxedStackFrame` must propagate the fallback filter registry from parent to child registers (see Section 3, `SandboxedStackFrame` propagation).

### 4. Add a feature-gated conformance module

- Behind `conformance-harness` (a feature on `liquid-core`), expose a `#[doc(hidden)] pub` `conformance` module in `liquid-core` and re-export through `liquid`.
- This module is explicitly unstable and not part of the supported production API.
- `crates/ruby-ext` is the only supported consumer by project policy.
- The module provides internal conformance parse and render hooks that `ruby-ext` uses alongside the real public `Template` entrypoints (`Template::render`, `Template::render_to`, `Template::render_with_options`, `Template::render_to_with_options`), without adding any public conformance-only method to `Template`.

### 5. Add hidden conformance parse and render entrypoints

- `ruby-ext` owns a conformance environment adapter that stores Ruby-registered filters/tags/blocks and feeds them into the conformance parse/render path.
- Ruby-defined tags/blocks are handled by registering them in the `PluginRegistry` before calling the core parser. This is built as part of the initial refactor, not deferred.
- Before parser construction, the conformance environment adapter compiles Ruby tag/block registrations into Rust-owned parser registration descriptors carrying stable handler IDs. Those descriptors, not Ruby objects, are what get registered in the `PluginRegistry`.

#### Prepass mechanism

- The prepass registers Ruby tag names as `ParseTag` (simple tags) or `ParseBlock` (block tags with bodies) implementations in the `ParserBuilder` before invoking the core parser. This uses the existing tag/block extension mechanism — no grammar changes, no string rewriting, no new parser hooks in core.
- The registered `ParseTag` / `ParseBlock` implementations are Rust-owned adapter wrappers around those descriptors, so the parser-side objects remain `Send + Sync` compatible.
- `ConformanceRubyBlockParser` implements `ParseBlock`: at parse time, it mirrors the current harness-side `Liquid::Block` behavior by validating top-level delimiters, collecting parsed child body nodes for `nodelist`, and compiling render fragments for default block rendering.
- `ConformanceRubyTagParser` implements `ParseTag`: at parse time, it captures the tag markup and returns a lightweight renderable for body-less custom tags.
- At parse time, these conformance parsers resolve the Ruby tag/block handler from the environment adapter and store a stable Rust-owned handler ID (or equivalent adapter handle) on the parsed conformance node.
- At render time, the parsed conformance node resolves that handler ID through the per-render conformance adapter table and delegates only the tag-specific behavior to Ruby.
- These `ParseTag`/`ParseBlock` implementations live in `ruby-ext`, not in core. Core doesn't change.

#### Constraints

- Tag/block registrations are snapshotted at parse time.
- Tag/block handler selection is the same across production and conformance: it is determined during parse, not by render policy.
- The parser-side registration objects are also Rust-owned adapter descriptors / wrappers, not direct Ruby objects. This keeps `ParseTag` and `ParseBlock` implementations `Send + Sync` compatible.
- The parse-time snapshot is a Rust-owned handler ID / adapter handle, not a direct Ruby object stored on the parsed node. This keeps parsed nodes `Send + Sync` compatible.
- Filter registrations remain environment-owned and, in conformance builds, resolve at render time via the `FallbackFilterRegistry`.
- Do not add a new generic parser hook to core. The prepass uses existing `ParseTag`/`ParseBlock` extension points.

### 6. Define what `RubyConformancePolicy` owns

- `RubyConformancePolicy` handles only adapter-specific behavior required for the Shopify Ruby suite:
  - Ruby exception-renderer behavior and error replay (via `handle_render_error`)
  - live-scope visibility for Ruby callbacks/drops during render
  - exact Ruby `ResourceLimits` compatibility (scoring, cumulative tracking, Ruby-visible mutation)
  - per-partial resource-limit reset timing (via `on_scope_push(true)` — the `partial` flag triggers reset)
- Ruby-defined filter dispatch is **not** a policy responsibility. Ruby filters dispatch through the `FallbackFilterRegistry` in registers (see Section 3a). The policy is not involved in filter dispatch.
- `RubyConformancePolicy` must not become a second engine semantics owner. Core Liquid behavior still lives in Rust engine code.

#### `RubyConformancePolicy` is a callback-based adapter defined in core

- `RubyConformancePolicy` is defined in `liquid-core` behind `#[cfg(feature = "conformance-harness")]` as a callback-based adapter struct. It holds closures/function pointers for error handling, resource-limit synchronization, and live-scope notifications.
- `ruby-ext` constructs a `RubyConformancePolicy` with Ruby-specific closures before each render and installs it in registers.
- The `RenderPolicy` trait remains sealed in core — `ruby-ext` configures instances of the conformance policy, it does not implement the trait itself.

### 7. Replace always-on Ruby session state with conformance-only compile-time gating

- Remove `LiveScopeSession` and Ruby session concepts from always-on core runtime/register state.
- `LiveScopeSession`, the `live_scope_session` field on `Registers`, and all `push_root_scope` calls in stdlib tags (`render_tag.rs`, `include_tag.rs`, `for_block.rs`) are gated behind `#[cfg(feature = "conformance-harness")]`.
- In production builds, the entire `LiveScopeSnapshot` construction and `push_root_scope` calls compile away. `Registers` drops the `live_scope_session` field. `SandboxedStackFrame::new` does not copy a session.
- `RubyConformancePolicy` receives live-scope events or equivalent internal notifications for:
  - iteration bindings
  - `forloop` / `forloop.parentloop`
  - partial-local assigns and aliases visible to Ruby callbacks
- These notifications are available only in conformance builds.
- Production builds compile none of this path.

### 8. Production public API: `RenderOptions` and `ErrorMode`

- Production keeps abuse protection, but not via the Ruby `ResourceLimits` API shape.
- All limit fields are `Option<usize>` where `None` means unlimited.
- Default `RenderOptions` has all limits set to `None` (unlimited) and uses `ErrorMode::default()`, which is lenient.
- Default `render()` and `render_to()` intentionally switch to Ruby-compatible lenient defaults for undefined variables and undefined filters. This is a compatibility-driven breaking change from the current Rust engine.
- Callers that want the current strict Rust behavior must opt in explicitly via `RenderOptions`.

#### `RenderOptions` struct

```rust
pub struct RenderOptions {
    /// Maximum total bytes written to output across the entire render tree.
    /// Enforced mid-write by CountingWriter. None = unlimited.
    pub max_output_bytes: Option<usize>,

    /// Maximum total render ops (AST nodes evaluated) across the entire render tree.
    /// None = unlimited.
    pub max_render_ops: Option<usize>,

    /// Maximum total bytes assigned/captured across the entire render tree.
    /// None = unlimited.
    pub max_assign_bytes: Option<usize>,

    /// Maximum combined scope + partial nesting depth.
    /// Matches Ruby Liquid's combined depth model (scope pushes + partial invocations).
    /// None = unlimited.
    pub max_depth: Option<usize>,

    /// Whether undefined variables produce an error (true) or return nil (false).
    /// Default: false (matches Ruby Liquid's default). Separate from error_mode —
    /// strict_variables controls whether a lookup *produces* an error,
    /// error_mode controls what *happens* when an error occurs.
    pub strict_variables: bool,

    /// Whether undefined filters produce an error (true) or pass input through unchanged (false).
    /// Default: false (matches Ruby Liquid's default).
    pub strict_filters: bool,

    /// Error handling mode: strict (abort on first error) or lenient (inline errors, continue).
    pub error_mode: ErrorMode,
}
```

- `strict_variables` and `strict_filters` are orthogonal to `error_mode`. All combinations are valid:
  - `strict_variables: false` + `ErrorMode::Strict` → undefined variables produce nil (no error), other errors abort.
  - `strict_variables: true` + `ErrorMode::Lenient` → undefined variables produce an error, but the error is rendered inline.
- Default `strict_variables: false` matches Ruby Liquid's default behavior. The current Rust engine defaults to strict (unknown variable → error). Changing the default to match Ruby is intentional and is part of the compatibility goal for this refactor.
- `strict_variables: false` is implemented as a runtime-level lenient lookup mode across the entire render tree, not just as a wrapper around the root globals object.
- The render entrypoint may still wrap the root globals in a lenient adapter, but that is not sufficient by itself. Stack-frame `get()` implementations (`StackFrame`, `GlobalFrame`, `IndexFrame`, `SandboxedStackFrame`) must also consult the active lookup mode and return nil for missing root variables instead of raising `Unknown variable`.
- The active lookup mode is installed per render and is visible through the runtime/register path so nested scopes, isolated `{% render %}` contexts, and other child frames all preserve the same `strict_variables` behavior.
- `strict_filters: false` is implemented after all filter-resolution paths are exhausted: when neither a conformance override nor a compiled/fallback dispatcher can resolve the filter, `filter_chain.rs` returns the input value unchanged instead of erroring.

#### `ErrorMode` enum

```rust
pub enum ErrorMode {
    /// Abort rendering on the first render error. Returns Err to caller.
    Strict,

    /// Continue rendering on errors. Write the formatted error string inline
    /// in the output and collect errors. The formatter function receives the
    /// error and returns the string to write.
    Lenient(fn(&Error) -> String),
}

impl ErrorMode {
    /// Lenient mode with the default Ruby-compatible formatter:
    /// "Liquid error (line N): message"
    pub fn lenient() -> Self {
        ErrorMode::Lenient(default_error_formatter)
    }
}
```

- `fn` pointer is used for the error formatter: zero-cost, `Copy`, `Send + Sync`, no allocation, no lifetime parameter on `RenderOptions`.
- The `Error` already carries template name, line number, and message — no captured state is needed.
- Both strict and lenient modes are supported in v1.

#### Render entrypoints and return types

```rust
/// Collected output and errors from a render_with_options call.
pub struct RenderOutput {
    output: String,
    errors: Vec<Error>,
}

impl RenderOutput {
    pub fn output(&self) -> &str { &self.output }
    pub fn into_output(self) -> String { self.output }
    pub fn errors(&self) -> &[Error] { &self.errors }
}

impl Template {
    /// Simple API: uses RenderOptions::default().
    /// This is intentionally Ruby-compatible lenient for undefined variables/filters.
    pub fn render(&self, globals: &dyn ObjectView) -> Result<String>;
    pub fn render_to(&self, writer: &mut dyn Write, globals: &dyn ObjectView) -> Result<()>;

    /// Advanced API: configurable limits and error mode.
    /// Returns RenderOutput with both the rendered string and collected errors.
    pub fn render_with_options(&self, globals: &dyn ObjectView, options: &RenderOptions) -> Result<RenderOutput>;
    /// Writer variant: returns collected errors (output written to writer).
    pub fn render_to_with_options(&self, writer: &mut dyn Write, globals: &dyn ObjectView, options: &RenderOptions) -> Result<Vec<Error>>;
}
```

- `render` / `render_to` use `RenderOptions::default()` (unlimited, Ruby-compatible defaults for undefined variables/filters, and lenient `ErrorMode::default()`). Return types unchanged, but behavior intentionally changes from the current Rust engine to match Ruby Liquid defaults.
- `render_with_options` returns `Result<RenderOutput>`:
  - **Strict, error:** `Err(error)`.
  - **Strict, no error:** `Ok(RenderOutput { output, errors: vec![] })`.
  - **Lenient:** `Ok(RenderOutput { output_with_inline_errors, errors: [collected] })`.
- `render_to_with_options` returns `Result<Vec<Error>>`:
  - **Strict, error:** `Err(error)`.
  - **Strict, no error:** `Ok(vec![])`.
  - **Lenient:** `Ok(vec![error1, ...])`.
- In lenient mode, errors appear both inline in the output (via the formatter) AND in the collected `errors` vec (for programmatic access). This matches Ruby where `template.render` returns inline errors in the string and `template.errors` provides the error objects.
- `Vec<Error>` doesn't allocate when empty, so strict mode callers pay nothing for error collection.
- The engine remains the only owner of production counter updates and boundaries.
- If internal scoring similar to `render_score` / `assign_score` is useful for implementation, it may exist internally, but it is not the public API.

### 9. Lock production limit semantics

- Production limits are cumulative across the entire render tree.
- `max_output_bytes` measures total bytes written across the whole render tree, including nested partials/renders/includes.
- `max_render_ops` measures total rendered-node/work count across the whole render tree, including nested partials/renders/includes.
- `max_assign_bytes` measures total assigned/captured bytes across the whole render tree, including nested partials/renders/includes.
- `max_depth` measures combined scope pushes + partial nesting depth, matching Ruby Liquid's `base_scope_depth + @scopes.length` model.
- Production limits do not reset at partial boundaries.
- Ruby reset semantics remain conformance-only.

#### Define render-op semantics

- One render op per `Renderable::render_to` invocation: each tag, output expression, or raw-text node counts as one op.
- The increment happens in the executor (wrapping each `render_to` call), not inside `Template::render_to` or inside individual renderables.
- For-loop iterations count: each iteration of a `for` block body is a separate executor invocation of the body's renderables, so iterations accumulate ops naturally.
- This means `max_render_ops` limits the total number of AST nodes evaluated, which is the best proxy for pathological input (e.g., a `for` over 10M items will count the loop body nodes × 10M iterations).

#### Define output-bytes enforcement

- `CountingWriter::write()` checks `max_output_bytes` on every write call and returns an `io::Error` immediately when the cumulative byte count exceeds the limit.
- The `io::Error` uses a sentinel `ErrorKind` (e.g., `WriteZero` or `Other`). `Template::render_to` maps it to a `liquid_core::Error` with an "output limit exceeded" message. Callers see a typed liquid error.
- This provides a hard limit, not an advisory check — a single expression that writes 100MB is stopped mid-write.
- `max_render_ops` and `max_assign_bytes` are checked by the executor at element boundaries (after each `render_to` call).
- `max_depth` is checked during stack frame construction (see Section 3), not at element boundaries.

#### Define depth enforcement

- `StackFrame::new` and `SandboxedStackFrame::new` increment the depth counter in registers during construction and decrement on drop (RAII). This matches Ruby's `check_overflow` being called inside `Context#push` and `Context#new_isolated_subcontext`.
- Depth increments occur on `StackFrame` creation (`for` loop iterations, `include` tag) and `SandboxedStackFrame` creation (`render` tag). `if` and `case` do not create new scopes and do not increment depth.
- When `max_depth` is exceeded during stack frame construction, a "nesting too deep" error is returned, matching Ruby's `StackLevelError`.
- This follows the same narrow-exception pattern as assign scoring (Section 3): stack frame construction interacts with limits through registers, but the executor owns all other limit checks at element boundaries.

### 10. Keep exact Ruby `ResourceLimits` only in conformance

- Full Ruby `ResourceLimits` semantics remain part of the conformance path only:
  - `render_score`
  - `assign_score`
  - `cumulative_render_score`
  - `cumulative_assign_score`
  - Ruby reset timing and Ruby-visible mutation during render
- `RubyConformancePolicy` mirrors exact engine deltas and reset boundaries into a Ruby-facing resource-limit facade/state view.
- `RubyConformancePolicy` does not own an independent scoring model; the engine remains the source of truth for render progress.
- Do not keep always-on Ruby-compatible resource state in production engine structs.

### 11. Reduce the Ruby harness to adapter/referee status

- Keep the Ruby API surface required to run the Shopify suite unchanged.
- Route as much real semantic behavior into Rust as the engine can express natively.
- Keep Ruby-side behavior only where the suite requires Ruby-defined extension points or Ruby-visible API shape.
- Treat the large Ruby compatibility shim as temporary scaffolding to reduce over time; it should adapt to Rust behavior, not permanently repair core engine behavior after the fact.

### 12. Migration strategy

- The refactor is executed as **one large change** (single PR).
- `DynamicFilterRuntime` in `ruby-ext` is decomposed in this change. It is deleted — no transitional dual-path state.
- The conformance harness must remain green (all Shopify suite tests passing) at the end of the PR.

#### `DynamicFilterRuntime` decomposition inventory

Every behavior in the current `DynamicFilterRuntime` (`ruby-ext/template.rs:677-1044`) and its helpers maps to a specific post-refactor home:

| Current behavior | Post-refactor home | Notes |
|---|---|---|
| Runtime delegation (partials, name, try_get, get, set_global, set_index, get_index, registers) | Data-only `Runtime` trait (unchanged) | These are data operations; they stay on `Runtime` |
| `set_global_alias` + `raw_assigns` caching of Ruby values | `Runtime::set_global_alias` stays on trait; raw Ruby value caching moves to conformance adapter in `ruby-ext` | Production `set_global_alias` works with native Rust values only |
| `roots()` merging `raw_assigns` keys | Conformance adapter wraps inner Runtime and adds conformance-tracked keys | Production roots() is unchanged |
| `try_host_filter` / Ruby filter dispatch | Conformance-only `FallbackFilterRegistry` in registers | Already specified (Section 3a) |
| `handle_render_error` full recovery logic | Split: production recovery via `ProdPolicy::on_render_error`; Ruby exception-renderer via `RubyConformancePolicy` callbacks | Production uses `ErrorMode` formatter; conformance uses Ruby `exception_renderer` |
| `increment_render_score` / `increment_assign_score` | `RenderPolicy::on_render_op` / `on_assign` | Already specified (Section 3) |
| `check_resource_limits` (write score) | `CountingWriter` + `RubyConformancePolicy` for Ruby `increment_write_score` | Production: CountingWriter enforces. Conformance: policy also mirrors to Ruby |
| `reset_resource_limits` | `RenderPolicy::on_scope_push(true)` | Already specified (Section 3) |
| `persistent_assigns` sync to Ruby hash | Conformance adapter in `ruby-ext` | Ruby-specific; `set_global` writes to both Rust state and Ruby hash |
| `resource_limits` Ruby object | `RubyConformancePolicy` state | Already specified (Section 10) |
| `RenderRecoveryState` / `exception_renderer` | `RubyConformancePolicy` callbacks | Conformance-only |
| `handled_errors` collection | `ProdPolicyState.errors` for production; conformance policy for Ruby | Both paths collect errors |
| `raised_exception` tracking | `RubyConformancePolicy` state | Conformance-only (Ruby exception re-raising) |
| `RenderRootObject` / tracked globals | Conformance adapter in `ruby-ext` | Variable lookup tracking for error recovery with Ruby exceptions |
| `LenientObject` wrapper for `strict_variables=false` | Production engine: runtime-level lenient lookup mode, with optional root-globals adapter as a helper | The behavior moves from `ruby-ext` into the engine, but the source of truth is the runtime-level lookup mode described in Section 8 |
| Error normalization (`normalize_error_message`, `RenderErrorMetadata`) | Stays in `ruby-ext` conformance shim | See Section 12a |
| `build_filter_host` / `collect_filter_modules` | Conformance-only `FallbackFilterRegistry` installation in `ruby-ext` render setup | Part of conformance render initialization |
| `LiveScopeSession` management | Conformance-only, feature-gated | Already specified (Section 7) |
| `to_liquid` / `liquid_method_missing` callbacks | Stays in `ruby-ext` `RenderDynamicObject` | Only needed for wrapping Ruby objects; not called from engine |

#### Error message normalization stays in `ruby-ext`

- The current codebase has parallel error normalization in both Rust (`normalize_error_message`, `RenderErrorMetadata::from_raw`) and Ruby (`Liquid::Error.classify_exception`, `Liquid::Error.extract_metadata`).
- Both apply identical mapping rules: "Unknown filter" → "undefined filter X", "Can't divide by zero" → "divided by 0", "Unknown variable" → "undefined variable X", etc.
- After the refactor, all error normalization stays in `ruby-ext` as a conformance shim. The Rust engine produces structured `Error` objects with metadata (template name, line number, context). The conformance shim in `ruby-ext` normalizes these into Ruby-compatible message formats.
- Production `ErrorMode::Lenient` formatter receives the raw `Error` with full metadata. The default formatter produces a standard format (`"Liquid error (line N): message"`). It does NOT apply Ruby-specific normalization. Users who want Ruby-compatible messages can provide a custom formatter.
- `RenderOutput.errors` preserves the full error chain: template name, line number, context keys, and original message. No information is lost.

### 12a. Prepass and raw blocks

- The prepass registers Ruby tag names from the environment's tag registry into the `PluginRegistry` — it does not scan template source. Raw blocks (`{% raw %}...{% endraw %}`) are handled naturally by the Pest parser's `Raw` grammar rule, which prevents content inside raw blocks from being parsed as tags. No special raw-block handling is needed in the prepass.

## What Ruby-Suite Parity Does Not Guarantee

- It does not guarantee identical performance characteristics to the Ruby implementation.
- It does not guarantee identical memory usage patterns or allocation behavior.
- It does not guarantee identical production API shape for features that Ruby Liquid does not expose as public API.
- It does not guarantee absence of bugs outside the coverage of the pinned upstream suite.
- It does not guarantee behavior for upstream revisions newer than the recorded pin.

## Thread Safety Model

- `Template` is `Send + Sync` (all fields are thread-safe). Templates can be shared across threads.
- Each `render()` / `render_with_options()` call creates its own `Runtime` with its own `Registers`. Concurrent renders of the same template from different threads are safe.
- A single render operation is single-threaded internally. `Registers` uses `RefCell` (not `Mutex`), which is appropriate because rendering is a sequential tree walk.
- `RenderOptions` is a plain struct with no generics or lifetimes. The `fn` pointer in `ErrorMode` is `Copy`, `Send`, `Sync`.
- The `FallbackFilterRegistry` and `RenderPolicy` stored in registers are per-render. They may be shared across nested renders within one render tree via `Rc`, but never across concurrent top-level renders.

## Test Plan

- Production lane:
  - root `cargo check`
  - root `cargo test`
  - both run with production `default-members` only and without Ruby installed
  - verify that `conformance-harness` feature is **not** enabled in production builds
- Conformance lane:
  - build `liquid_ext` with `conformance-harness`
  - run `make harness-test`
  - this remains required for compatibility-affecting changes
- Add Rust-native tests before deleting harness-owned behavior for:
  - `render` / `include`
  - `forloop` and nested loops
  - parser quirks and strict/lax modes
  - strict vs non-strict render behavior
  - production safety-limit enforcement for `max_output_bytes`, `max_render_ops`, `max_assign_bytes`, and `max_depth`
  - partial caching and related built-in semantics
- Add executor/policy tests for:
  - executor calls `on_render_op` once per `Renderable::render_to` invocation, not once per template body
  - `CountingWriter` enforces `max_output_bytes` mid-write (not just at element boundaries)
  - `CountingWriter` io::Error maps to liquid error with "output limit exceeded"
  - `max_render_ops` and `max_assign_bytes` are checked at element boundaries by the executor
  - `max_depth` enforced during `StackFrame::new` and `SandboxedStackFrame::new`, returns "nesting too deep" error
  - depth counter decrements on stack frame drop (RAII)
  - `if` and `case` blocks do not increment depth
  - `assign_tag` and `capture_block` report assign sizes through policy in registers
  - `ProdPolicy` strict mode aborts on first error
  - `ProdPolicy` lenient mode writes formatted error inline and continues
  - lenient mode collects errors in `RenderOutput.errors` AND writes them inline in output
  - `render_with_options` returns `Result<RenderOutput>` with both output and collected errors
  - `render_to_with_options` returns `Result<Vec<Error>>` with collected errors
  - custom error formatter via `ErrorMode::Lenient(fn)` is called correctly
  - `strict_variables: false` applies a runtime-level lenient lookup mode so missing variables resolve to nil across the whole render tree, including nested and isolated scopes
  - `strict_variables: true` + `ErrorMode::Lenient` produces inline error for undefined variables
  - `strict_filters: false` returns input unchanged for undefined filters (no error)
  - `strict_filters: true` + `ErrorMode::Lenient` produces inline error for undefined filters
  - `RenderOutput.errors` preserves full error chain (template name, line number, context)
  - `ProdPolicy` never resets limits at partial boundaries
  - `RubyConformancePolicy` resets limits at partial boundaries per Ruby semantics
  - policy is installed in registers before render and removed after
  - `SandboxedStackFrame` propagates policy, fallback filter registry, and adapter registers from parent (policy works correctly inside `{% render %}` partials)
- Add fallback filter registry tests for:
  - late-registered filters dispatch through conformance-only `FallbackFilterRegistry` in registers
  - `filter_chain.rs` evaluates argument expressions before calling fallback dispatch (no `ParseFilter::parse()` step)
  - production deferred filter with `strict_filters: false` returns input unchanged without a runtime dispatcher
  - production deferred filter with `strict_filters: true` produces "unknown filter" error without a runtime dispatcher
  - conformance deferred filter with no fallback registry produces "unknown filter" error when strict
  - `SandboxedStackFrame` propagates fallback filter registry from parent
  - Ruby filter adapters dispatch through fallback registry in conformance builds
- Add conformance-policy tests for:
  - exception renderer replay through `RubyConformancePolicy`
  - live-scope visibility during loop/render/include callbacks
  - exact Ruby `ResourceLimits` synchronization through conformance-only state/facade
  - conformance prepass registers Ruby tags/blocks as `ParseTag`/`ParseBlock` in `PluginRegistry` before parsing
  - custom Ruby blocks build parsed body/nodelist data and compiled render fragments during parse
  - custom Ruby blocks preserve delimiter validation and Ruby-compatible invalid-delimiter errors
  - prepass uses existing extension mechanism — no core grammar changes
  - raw blocks handled naturally by Pest parser (prepass registers from environment, does not scan template source)
  - error normalization stays in `ruby-ext` conformance shim, not in engine
  - feature-off builds proving the conformance path is absent and production still compiles cleanly
  - `LiveScopeSession`, `push_root_scope`, and `Registers.live_scope_session` are absent from production builds
- Acceptance criteria:
  - production builds/tests do not pull Ruby, `magnus`, or conformance-only code paths
  - `Runtime` trait has no scoring, error-handling, resource-limit, or filter-override methods
  - `render_tag.rs` and `include_tag.rs` contain no direct `reset_resource_limits` calls — replaced with `on_scope_push(true)` through the policy in registers
  - three narrow exceptions access the policy through registers: (1) `assign_tag` and `capture_block` for assign scoring, (2) `StackFrame`/`SandboxedStackFrame` construction for depth tracking, (3) `include`/`render` tags for partial-entry hook. All other renderables use `Runtime` for data only
  - `filter_chain.rs` has no `runtime.evaluate_filter` calls
  - production has no runtime-installable filter-dispatch hook
  - `FallbackFilterRegistry` exists only in conformance builds
  - `Registers` has no `live_scope_session` field in production builds
  - `RenderPolicy` trait is sealed — `ruby-ext` configures `RubyConformancePolicy` instances but does not implement the trait
  - production exposes `RenderOptions` with `ErrorMode::Strict` and `ErrorMode::Lenient`, `strict_variables`, and `strict_filters` — all work in v1
  - `strict_variables: false` is implemented by a production-engine runtime lookup mode, not only by a root-globals wrapper
  - error message normalization stays in `ruby-ext`, not in the production engine
  - full Ruby harness still passes through the explicit conformance lane
  - the only supported conformance-policy consumer is `crates/ruby-ext`

## Production Quality Strategy

- Ruby-suite parity is necessary but not sufficient for production confidence.
- Production correctness must also be protected by Rust-native tests for engine-owned behavior, especially:
  - explicit safety limits
  - concurrency and shared-`Template` rendering
  - parser/render edge cases not directly covered by the upstream suite
  - partial/render recursion and isolation behavior
- Add parser and render fuzzing for crash safety and malformed-input resilience.
- Keep differential or golden tests for high-risk semantics where regressions are cheap to encode in Rust after upstream behavior is confirmed.
- Production bugs are minimized by combining:
  - full conformance-lane coverage against the pinned upstream suite
  - focused Rust-native regression tests
  - fuzzing and concurrency verification
  - explicit production safety-limit tests

## Production Ship Criteria

- The full pinned upstream Ruby suite is green through the conformance lane.
- The production build/test lane is green with `conformance-harness` disabled.
- There are no known unresolved P0/P1 behavioral diffs against the pinned upstream suite.
- Production safety-limit behavior is covered by Rust-native tests.
- Shared-`Template` concurrency tests exist and pass.
- Parser/render fuzzing is part of the default pre-release quality bar.
- Fuzzing may be waived for a release only by an explicit release decision that is recorded together with the reason for the waiver and the intended follow-up to restore fuzz coverage.
- Conformance-only crates/features are absent from production builds.
- Any production-only feature added after conformance parity is either:
  - disabled by default, or
  - proven not to change the default Ruby-compatible behavior claimed by this spec.

## Assumptions / Defaults

- Greenfield assumption: no external consumers depend on the current `Runtime` surface or on any internal conformance-only render hooks, so breaking cleanup is acceptable.
- Production only needs Rust-native extensibility; it does not need Ruby-style dynamic host callbacks or foreign-language object semantics.
- Full Ruby conformance coverage remains required.
- One workspace with strict `default-members` and conformance-only features is the starting design; a second workspace is deferred unless isolation still proves insufficient.
- The design follows the same pattern as Ruby Liquid: tags receive a data-access context (`&dyn Runtime`), and the executor/render-loop owns all policy calls (scoring, error handling, resource limits, filter dispatch). This is not a novel abstraction — it matches the existing Ruby architecture where `Context` is used by tags for data only and `BlockBody` / `Template` own the control flow.
- `RenderOptions` defaults to unlimited limits, `strict_variables: false`, `strict_filters: false`, and `ErrorMode::default()` (lenient). The `strict_variables`/`strict_filters` defaults intentionally match Ruby Liquid, even though this changes the current Rust engine's default behavior.
- `Template` is `Send + Sync`. Each render creates its own `Runtime` and `Registers`. Concurrent rendering of the same template from different threads is safe.
