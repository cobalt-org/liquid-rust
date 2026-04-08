# Data Model: Pure-Rust Production Engine with Ruby-Only Conformance Harness

## 1. `RenderOptions` (public)

Configuration object for a single top-level render.

| Field | Type | Default | Notes |
|---|---|---|---|
| `max_output_bytes` | `Option<usize>` | `None` | Hard output cap enforced mid-write by `CountingWriter` across the full render tree. |
| `max_render_ops` | `Option<usize>` | `None` | Cumulative count of renderable executions across nested templates, partials, and loop iterations. |
| `max_assign_bytes` | `Option<usize>` | `None` | Cumulative assigned/captured bytes across the full render tree. |
| `max_depth` | `Option<usize>` | `None` | Combined scope and partial depth across the render tree. |
| `strict_variables` | `bool` | `false` | Missing variables resolve to `nil` when `false`; applies across nested and isolated scopes. |
| `strict_filters` | `bool` | `false` | Unknown filters pass input through unchanged when `false`. |
| `error_mode` | `ErrorMode` | `ErrorMode::Lenient(default formatter)` | Default public rendering is lenient per the clarified feature spec. |

**Validation rules**

- `None` means unlimited for all numeric limits.
- Limit values are copied into per-render policy state at render start.
- `strict_variables` and `strict_filters` are orthogonal to `error_mode`.
- `RenderOptions::default()` must be semantically equivalent to the default `Template::render` and `Template::render_to` entrypoints.

## 2. `ErrorMode` (public)

Controls what the engine does after a render error is produced.

| Variant | Payload | Behavior |
|---|---|---|
| `Strict` | none | Abort immediately and return `Err`. |
| `Lenient` | `fn(&Error) -> String` | Write formatted inline output, continue rendering, and collect the original `Error`. |

**Validation rules**

- The formatter is a function pointer so `RenderOptions` stays non-generic and lifetime-free.
- Formatter output is written inline only in lenient mode; raw `Error` values are still preserved for collection.

## 3. `RenderOutput` (public)

Result type for configurable render entrypoints.

| Field | Type | Source | Notes |
|---|---|---|---|
| `output` | `String` | Rendered writer buffer | Includes inline formatted errors in lenient mode. |
| `errors` | `Vec<Error>` | Drained from per-render state | Empty on successful strict renders; populated on lenient renders with recoverable errors. |

**Relationships**

- Returned by `render_with_options`.
- Writer-based `render_to_with_options` returns only the collected `Vec<Error>` because the caller already owns the output sink.

## 4. `ProdPolicy` and `ProdPolicyState` (internal)

Internal production policy installed per top-level render.

### Immutable policy configuration

| Field | Type | Source |
|---|---|---|
| `max_render_ops` | `Option<usize>` | `RenderOptions` |
| `max_assign_bytes` | `Option<usize>` | `RenderOptions` |
| `max_depth` | `Option<usize>` | `RenderOptions` |
| `error_mode` | `ErrorMode` | `RenderOptions` |

### Shared mutable state

| Field | Type | Lifecycle |
|---|---|---|
| `render_ops` | counter | Starts at zero, increments before each renderable execution. |
| `assign_bytes` | counter | Starts at zero, increments from `assign`/`capture` sites. |
| `depth` | counter | Starts at zero, increments/decrements on scope push/pop. |
| `errors` | `Vec<Error>` | Empty at start, appended in lenient mode, drained on successful completion. |

**Validation rules**

- Created fresh for every top-level render.
- Shared across nested partials and nested `{% render %}` calls.
- Never shared across concurrent renders of the same `Template`.
- Does not own `max_output_bytes`; output-byte enforcement stays in `CountingWriter`.

**State transitions**

1. `Initialized` from `RenderOptions`
2. `Installed` into registers at the entrypoint
3. `Mutating` during rendering as hooks update counters/errors
4. `Drained` into `RenderOutput` on success, or discarded on strict failure

## 5. `RubyConformancePolicy` and Conformance Adapter State (internal, feature-gated)

Feature-gated state used only when `conformance-harness` is enabled.

| Component | Purpose |
|---|---|
| `RubyConformancePolicy` | Preserves Ruby-specific resource-limit and error-handling behavior for the harness path. |
| Fallback filter dispatcher | Resolves late-registered Ruby filters and render-time overrides. |
| Live scope session | Tracks Ruby-visible scope behavior for conformance tests that inspect scope activity. |
| Ruby callback/adapter handles | Bridge policy and parser/render events back into `crates/ruby-ext`. |

**Validation rules**

- None of these handles may compile into the default production build.
- Ruby-specific error normalization stays in `crates/ruby-ext`, not in production engine code.
- Conformance state must propagate into isolated render scopes created by `SandboxedStackFrame`.

## 6. Register-Carried Shared Handles (internal)

Registers remain the transport layer for per-render shared state.

| Handle | Shared by normal stack frames | Propagated into `SandboxedStackFrame` |
|---|---|---|
| Interrupt state | Yes | No, remains isolated |
| Active render policy | Yes | Yes |
| Fallback filter dispatcher | Yes when present | Yes |
| Live scope / conformance adapter state | Yes when present | Yes |

**Validation rules**

- Normal stack frames forward parent registers.
- `SandboxedStackFrame` creates a new register bag, but must copy only the shared handles needed for conformance and production policy continuity.
- Shared handles are propagated by reference/handle, never by duplicating mutable state.

## Relationships Summary

- `Template::render*` entrypoints create `RenderOptions`-derived policy state and install it into registers.
- The executor consumes policy hooks on every renderable boundary.
- Narrow exception sites (`assign`, `capture`, scope push/pop) update the same policy state from inside renderables and stack-frame constructors.
- `RenderOutput` drains collected errors from the production state after a successful configurable render.
- The conformance bridge installs additional register-carried handles that the hidden `liquid`/`liquid-core` conformance module and `crates/ruby-ext` share.
