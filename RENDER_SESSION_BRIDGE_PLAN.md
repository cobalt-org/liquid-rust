# Explicit Render-Session Bridge Plan

## Summary

Replace the current thread-local live-scope bridge with an explicit per-render session owned by the Rust runtime and shared through a render-lifetime native session reference on the Ruby context handle. Restore the Ruby harness `Liquid::Context` behavior close to upstream Shopify Liquid so the bridge only fills the missing runtime gap: live Rust-only root bindings such as `forloop`, current iteration bindings, and partial-local assigns visible during callbacks from Ruby drops/procs.

This plan is intentionally narrower than the broader Shopify Ruby harness plan. It is focused on removing the `thread_local` workaround, making render-local scope propagation explicit, and avoiding further drift in the Ruby compatibility layer.

## Motivation

### Triggering compatibility bug

- `crates/ruby-ext/src/values.rs:473-487`
- Enumerable drops with `each` but no `to_a` no longer iterate.
- Shopify's `EnumerableDrop` fixture intentionally defines `each`/`count`/`first` without `to_a`.
- Current compatibility fixes only work for enumerable Ruby objects that also expose `to_a`.
- The underlying architectural issue is that render-time state and loop-local visibility are currently patched through ad hoc bridging instead of a render-local source of truth.

### Problems with the current thread-local bridge

- It hides runtime state in ambient process state instead of in the render session.
- It works only because Ruby callbacks happen on the same OS thread as the Rust render path.
- It is hard to reason about with nested renders and future execution-model changes.
- It encouraged reimplementing `Liquid::Context` behavior in Ruby instead of restoring upstream semantics.

## Review Findings To Preserve

### Findings from plan review

1. `Context#scopes` must stay compatible with Shopify Liquid.
   - The prior plan proposed exposing synthetic read-only live frames through `scopes`.
   - That is too ambiguous and likely wrong because Shopify callers and tags use `scopes.last[...] = ...`.
   - The plan must preserve `scopes` as the writable Ruby scope stack and avoid making synthetic live frames look like writable Ruby scopes.

2. The render-session ownership and propagation rule must be explicit.
   - The prior plan did not specify how `Context`, `new_isolated_subcontext`, `stack`, `render`, and `include` keep pointing at the same live session.
   - The owning object and propagation rules must be fixed up front so the implementer does not invent incompatible handle lifetimes.

3. The lookup boundary between Ruby and Rust must be explicit.
   - The prior plan restored upstream `Context#[]` while also proposing native live-scope lookup, but it did not say exactly where live render scopes are consulted.
   - The implementer must not be left to choose between root-only lookup in Ruby and full native path resolution.

## Final Design Decisions

### 1. Render-session ownership

- The render session is owned by the Rust render runtime, not by Ruby `Context`.
- A `LiveScopeSession` object is created once per top-level render entrypoint in the Rust extension.
- The render entrypoint also creates a native `RenderSessionRef` handle that points at that same session.
- The session must be stored behind explicit shared ownership (`Rc`/`Arc` or equivalent stable heap ownership), not as a borrowed pointer into stack-local runtime state.
- The runtime stores the shared session owner in `Runtime::registers()`, and the Ruby context handle stores a clone/wrapper `RenderSessionRef` for the duration of the active render only.
- `RenderSessionRef` must carry enough identity/state to distinguish an actively running render tree from a stale ref retained on an old child context after render completion.
- Nested `render` and `include` operations reuse the existing session/ref pair from the active runtime instead of creating a new one.
- Ruby `Context` objects do not own the live session; they only carry the render-lifetime `RenderSessionRef` while a render is active.
- Any runtime frame type that intentionally sandboxes variable lookup but still participates in the same render tree must continue to share the same live session; sandboxing variable visibility must not imply a fresh live session.

### 2. Session propagation

- The native template/context render entrypoint creates or reuses the render session before entering Rust rendering.
- On top-level render, the entrypoint installs `RenderSessionRef` onto the context handle before rendering and removes it in an ensure/finally path after render completion.
- On nested render paths, the entrypoint reuses the already-installed `RenderSessionRef` only if it is still active for the current render tree.
- A merely present but inactive/stale ref on a retained child context must not cause a later top-level render to be treated as nested reuse.
- The render entrypoint must track whether it installed the ref or merely reused an existing one; only the installing frame is allowed to remove it on exit.
- The native context helpers used during Ruby callbacks resolve the live session through `RenderSessionRef`, not through ambient thread state.
- The native context handle passed to Ruby callbacks must continue to refer to the same runtime-backed session for:
  - the initial render
  - nested partial renders
  - `Context#stack`
  - `Context#new_isolated_subcontext`
- Any new `Context` created while a render is active, including `new_isolated_subcontext`, must inherit the current `RenderSessionRef` into its freshly allocated native handle.
- For implementability, this guarantee applies to Liquid-managed child context construction paths, such as template render helpers and `new_isolated_subcontext`, not to arbitrary external Ruby calls that allocate unrelated `Context` instances with no parent/session argument.
- That inheritance must happen through explicit constructor arguments or explicit native-handle propagation from the caller; it must not rely on hidden globals or thread-local lookup.
- `new_isolated_subcontext` creates a new Ruby-visible scope stack and register view, but it still points at the same active render session while it is part of the same render tree.
- Outside an active render, live session queries return no live-scope values and scope depth `0`.
- When a render tree ends, any retained child contexts may still physically hold the old `RenderSessionRef`, but that ref must become inert so future renders treat it as absent unless a new active render explicitly installs/replaces it.

### 3. Live-scope producers

- Push every Rust-only root frame that Ruby callbacks must be able to observe during the active render.
- `crates/lib/src/stdlib/blocks/for_block.rs` pushes a snapshot of the current iteration scope before rendering the loop body and pops it via RAII guard after the iteration completes.
- `crates/lib/src/stdlib/tags/render_tag.rs` pushes the partial-local root frame created for `render`, including `forloop`, the loop variable for `render ... for`, and any `with` / named arguments, and pops it after the partial body returns.
- `crates/lib/src/stdlib/tags/include_tag.rs` pushes the pass-through locals created for `include` and pops them after the partial body returns.
- Each pushed snapshot must include the data needed by Ruby-visible callbacks:
  - current loop variable binding
  - `forloop` object/value
  - any partial-local root names already visible in Rust during that render step
- Do not push Ruby-owned mutable scopes that already live in `Context#scopes`; this session is only for Rust-only runtime frames.

### 4. `Liquid::Context` compatibility rule

- Restore `harness/ruby-liquid/lib/liquid/context.rb` to upstream Shopify semantics as closely as practical for:
  - initializer arity and defaults
  - `@scopes`, `@environments`, and `@static_environments`
  - `push` / `pop` and base-scope guard
  - `[]` using upstream expression parsing/evaluation flow
  - `find_variable` precedence and contextual-drop handling
- `Context#scopes` remains the real mutable Ruby scope stack only.
- Live render scopes are not appended to `scopes` and are not exposed as writable Ruby hashes.
- Do not virtualize `Context#scopes` with a proxy/wrapper for loop depth. If a compatibility surface needs live depth, expose it through an explicit native helper instead of changing `scopes`.

### 5. Lookup boundary between Ruby and Rust

- Keep upstream Ruby `Context#[]` and expression evaluation semantics.
- Do not move full expression parsing into Rust in this change.
- Live render scope consultation happens only as a root-name override step.
- Concretely:
  - Ruby `Expression.parse` / `VariableLookup` stays responsible for parsing expressions.
  - Ruby `Context#find_variable` keeps the upstream Ruby lookup order across scopes, environments, static environments, and contextual-drop behavior.
  - During that Ruby lookup flow, a narrow native helper is consulted only to ask whether the active render session has a live override for the requested root name.
  - If the helper returns a live value, Ruby uses that as the resolved root.
  - If the helper reports no live override, Ruby continues the normal upstream Ruby fallback path unchanged.
  - After the root value is found, normal Ruby/Shopify path traversal semantics continue from that value.
- Add a native helper for live scope depth reporting if a compatibility surface requires loop-depth introspection, but do not use it to replace general expression evaluation.

### 6. Removal of thread-local bridge

- Remove any remaining live-scope `thread_local!` storage and helper callers.
- Replace those call sites with render-session-aware register access.
- The finished design must not depend on callback execution staying on the same OS thread.

## Implementation Plan

### Rust runtime

- Add `LiveScopeSession` under `crates/core/src/runtime`.
- Add a small native `RenderSessionRef` wrapper type in the Ruby extension for passing the active session through the Ruby context handle during render.
- `RenderSessionRef` must own a shared reference to heap-stable session storage; it must not borrow from a temporary `Registers::get_mut()` guard or any stack-local runtime frame.
- `RenderSessionRef` must expose an active render-tree identity or liveness check so callers can distinguish reusable in-tree refs from stale refs retained after a prior render ended.
- Store the session in `Registers` as render-local state.
- Update any runtime frame that currently allocates fresh registers for isolation, including `SandboxedStackFrame`, so it still shares or explicitly carries the active live session for the duration of the render tree.
- Define an owned live-scope snapshot representation for session entries. It must be able to capture the current root bindings without storing borrowed `&dyn ValueView` references from stack-local maps.
- That snapshot representation must preserve Ruby-backed dynamic values needed for callback correctness; do not eagerly flatten everything into plain scalars/objects if doing so would change drop/proc semantics.
- `LiveScopeSnapshot` must also support owned representation of transient Rust-local loop values such as `forloop`, including any visible `forloop.parentloop` chain.
- Snapshotting loop state must not retain borrowed `&dyn ValueView` links from the active runtime; nested loop metadata must be copied or otherwise stored in heap-stable owned form.
- Provide methods:
  - `push_root_scope(scope: LiveScopeSnapshot) -> guard`
  - `find_root(name: &str) -> Option<LiveScopeValue>`
  - `depth() -> usize`
- Keep the API root-oriented for this change; path traversal remains outside the session API.

### Rust stdlib integration

- Update `crates/lib/src/stdlib/blocks/for_block.rs` to build owned loop-local snapshots and push them into the active session for each iteration.
- Update `crates/lib/src/stdlib/tags/render_tag.rs` to build owned partial-local snapshots and push them into the active session for the duration of each partial render.
- Update `crates/lib/src/stdlib/tags/include_tag.rs` to build owned include pass-through snapshots and push them into the active session for the duration of each partial render.
- Ensure `{% render %}` continues to see the same active live session even though it uses a sandboxed runtime frame for variable lookup isolation.
- Ensure guard-based cleanup works under normal render, `break`, `continue`, and render errors.

### Ruby extension bridge

- Update the render entrypoints in `crates/ruby-ext/src/template.rs` to install/remove `RenderSessionRef` on the Ruby context handle around the active render.
- Reuse an existing `RenderSessionRef` only when it is still active for the current render tree; otherwise allocate/install a fresh top-level session.
- Make the render entrypoint record whether it installed or reused the ref so nested renders cannot clear a parent render's session on unwind.
- Update `crates/ruby-ext/src/context.rs` so the native helper resolves live roots through `RenderSessionRef` and converts `LiveScopeValue` back into the Ruby-visible value form expected by upstream `Context`.
- Extend the native context-construction path so Liquid-managed callers can explicitly pass an existing `RenderSessionRef` when constructing a new context/native handle during an active render.
- Keep separate native helpers for:
  - live root override lookup
  - live scope depth
- The live root helper must not reimplement Ruby fallback lookup, proc caching, or expression traversal.
- Do not add a general-purpose native expression parser/evaluator here.

### Ruby harness compatibility

- Restore `harness/ruby-liquid/lib/liquid/context.rb` toward upstream behavior.
- Remove ad hoc lookup parsing that was introduced to compensate for the thread-local bridge.
- Preserve upstream API shape for constructor, scope mutation, and environment precedence.
- Preserve the public Ruby `Context` API shape while allowing an internal-only path for Liquid-managed callers to pass a parent handle or `RenderSessionRef` when constructing child contexts during an active render.
- Ensure Liquid-managed `Context` construction during an active render inherits the currently installed `RenderSessionRef` into the child native handle via explicit argument/parent-handle propagation from the caller.
- Ensure `new_isolated_subcontext` preserves the active render-session ref while still creating an isolated Ruby-visible scope stack and register view.
- Ensure helper constructors used by template rendering paths also thread that ref through when they allocate fresh contexts during an active render.
- Keep the bridge integration limited to a narrow live-root override query and any minimal depth query needed for compatibility assertions.

## Test Plan

### Targeted Shopify Ruby tests

- `make harness-test TEST=test/integration/drop_test.rb`
- `make harness-test TEST=test/integration/context_test.rb`
- `make harness-test TEST=test/integration/tags/render_tag_test.rb`
- `make harness-test TEST=test/integration/tags/include_tag_test.rb`

### Required scenarios

- Enumerable drops with `each` but no `to_a` iterate correctly in `{% for %}`.
- Drops and procs can observe `forloop.index` and current loop-local bindings during render callbacks.
- Drops and procs invoked inside nested loops can observe both `forloop.index` and `forloop.parentloop.index` during the active iteration.
- Drops and procs invoked from inside `render` / `include` partials can observe partial-local assigns that exist only in Rust runtime frames.
- `Context#[]` handles ranges, hash-only access, and upstream variable lookup semantics again.
- `Context#push` / `pop` preserve the base-scope guard and raise `Liquid::ContextError` on extra `pop`.
- Nested partial renders share the same live render session for the duration of the render tree.
- Separate top-level renders do not leak live-scope state to each other.
- A child context retained from one completed render does not cause a later top-level render to reuse the stale session ref.

### Rust-side tests

- Add unit tests for `LiveScopeSession`:
  - nested push/pop ordering
  - empty session behavior
  - no leakage after guard drop
- Add integration coverage for nested `for` rendering where loop-local values are visible to Ruby callbacks during the active iteration only.
- Add integration coverage for nested loops where Ruby callbacks can observe `forloop.index` and `forloop.parentloop.index` through the live-session bridge.
- Add integration coverage for `render` / `include` partials where Ruby callbacks can observe partial-local Rust-only roots during the partial render and cannot observe them after unwind.
- Add integration coverage for nested render entry/exit so an inner render that reuses the existing session ref does not clear the parent's active session on return.
- Add integration coverage for `{% render %}` specifically verifying that sandboxed runtime isolation does not break live-session visibility for Ruby callbacks.
- Add integration coverage for fresh `Context` construction during an active render verifying that the child context inherits the active session ref without any ambient global bridge.
- Add integration coverage for a retained child context from a completed render verifying that a later top-level render installs a fresh active session instead of reusing the stale ref.
- Add unit/integration coverage for snapshot conversion so loop/render/include producers can store owned live-scope values without losing Ruby-backed dynamic lookup behavior needed by callbacks.

## Acceptance Criteria

- No remaining production code depends on `thread_local!` live-scope storage.
- The render session is explicit, render-local, and reused across nested renders in the same render tree.
- Ruby callbacks resolve live Rust-only roots through the explicit render-session ref, not through ambient process state.
- Liquid-managed child contexts created during an active render inherit the same active render-session ref, and nested render entrypoints do not clear refs they did not install.
- Stale refs retained on child contexts after render completion are inert and cannot cause future top-level renders to reuse the wrong session.
- Sandboxed render frames preserve the active live session even when they isolate normal variable lookup and allocate otherwise-independent runtime state.
- Live-scope session entries use owned snapshots rather than borrowed stack references, without regressing Ruby-backed dynamic callback behavior.
- `Context` behavior is closer to upstream Shopify Liquid, not farther away.
- The targeted upstream Ruby tests above pass.
- The full default harness command still passes:
  - `make harness-test`

## Assumptions

- This change does not attempt to move all `Liquid::Context` evaluation into Rust.
- This change does not broaden live-scope tracking beyond currently required loop-local state unless a failing upstream test proves that necessary.
- Preserving upstream Ruby `Context` semantics is a required part of this refactor, not optional cleanup.
