# Architectural Decision Log

Every decision made during the design of the production/conformance split plan, organized by category.

## High-Level Architecture

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| Production engine direction | Pure Rust, no Ruby in production | Keep Ruby hooks in production | No external consumers depend on current Runtime hooks; greenfield cleanup is acceptable |
| Ruby's role | Conformance harness only | Remove Ruby entirely; keep Ruby in production | Need to keep Shopify suite green, but Ruby should not shape production architecture |
| Workspace structure | Single workspace with `default-members` | Two separate workspaces | Simpler to maintain; revisit if isolation proves insufficient |
| Conformance feature gate | `conformance-harness` Cargo feature on `liquid-core` | Feature on `liquid` only; `cfg` flag via RUSTFLAGS | Feature on core lets `Registers`, stack frames, and stdlib tags use `#[cfg]` gating directly |
| Feature leak concern | Not concerned | Add compile-time assertions; use cfg flag instead | `conformance-harness` is opt-in, clearly named, and `doc(hidden)`. Accidental enablement is additive, not breaking |

## Runtime Trait & Policy

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| `Runtime` trait scope | Data-only (partials, get/set, registers) | Keep policy hooks on Runtime; split into two traits | Matches Ruby Liquid where tags use Context for data only; policy calls are executor's job |
| `Renderable::render_to` signature | Keep `&dyn Runtime` unchanged | Add policy parameter; make generic over policy | Changing the trait would touch every tag/block implementation; unnecessary since policy routes through registers |
| Policy abstraction | Internal sealed `RenderPolicy` trait | Generic hook bag; no abstraction (hardcode behaviors) | Sealed trait prevents external implementations; two policy impls cover all use cases |
| Policy storage mechanism | Stored in `Registers` (anymap) | Separate field on Runtime; thread-local; passed as parameter | Registers already exist, accessible at every recursion depth via `runtime.registers()`, no trait changes needed |
| `SandboxedStackFrame` register isolation | Propagate policy + adapter state from parent to child registers | Share parent registers; change to forwarding | Follows existing pattern (already copies `live_scope_session`); keeps interrupt isolation intact |
| Policy implementations | `ProdPolicy` (strict + lenient) and `RubyConformancePolicy` | Single policy with modes; three separate policies | Two policies cleanly separate production from conformance; strict/lenient is a mode within ProdPolicy |
| `RubyConformancePolicy` location | Defined in `liquid-core` behind feature gate as callback-based adapter | Defined in `ruby-ext`; defined without callbacks | Sealed trait prevents external impls; callbacks let `ruby-ext` configure without implementing the trait |

## Executor Design

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| Executor identity | `Template::render_to` becomes the executor | Separate executor function; executor struct | Template::render_to is already the render loop; container tags call it on child Templates, so recursion works naturally |
| Who calls policy hooks | Executor wraps each `render_to` call | Individual renderables call hooks; hooks on Runtime trait | Matches Ruby where BlockBody owns the control flow, not individual tags |
| Assign scoring exception | `assign_tag` and `capture_block` access policy through registers | Executor measures externally; keep on Runtime trait | Assign sizes are known only inside these two tags; executor can't measure externally |
| `reset_resource_limits` at partial boundaries | Policy-dispatched by executor | Tags call reset directly; always reset; never reset | ProdPolicy never resets (cumulative); RubyConformancePolicy resets per Ruby semantics |

## Filter Dispatch

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| Filter dispatch location | Stays in `filter_chain.rs`, not executor | Move to executor boundary; move to policy | Filters are evaluated inside Variable::render_to during expression evaluation; executor can't intercept |
| `runtime.evaluate_filter` | Removed from Runtime trait | Keep as non-overridable; keep as overridable | Override hook is the coupling point; removal eliminates the DynamicFilterRuntime pattern |
| `evaluate_filter_with_registry` visibility | Made internal (not public API) | Keep public; make pub(crate) | Production callers never need to call this directly |
| Late-registered filters | Supported through a hidden conformance-only fallback resolver in registers | Production support; patch compiled AST | The supported post-parse registration story is the Ruby harness path. Production keeps the stable Rust `ParserBuilder` registration model |
| Fallback filter dispatch type | Hidden callback trait stored in registers | Same `PluginRegistry<Box<dyn ParseFilter>>` type; pre-built `Box<dyn Filter>` | Avoids reintroducing public runtime interposition while still letting `ruby-ext` resolve late filters by name at render time |
| `FallbackFilterRegistry` availability | `conformance-harness` only | All builds; production-facing API | The fallback resolver exists to keep Ruby compatibility machinery out of the production surface |

## Public API: `RenderOptions`

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| Limit field types | `Option<usize>`, `None` = unlimited | Plain `usize` with `MAX` sentinel; dedicated `Limit` enum | Explicit, no ambiguity between "unlimited" and "forgot to set" |
| Default limits | All `None` (unlimited) | Conservative defaults (e.g., 10MB output) | Matches current behavior; users opt into limits explicitly; least surprising for existing callers |
| Error handling modes | Both strict and lenient in v1 | Strict only in v1, add lenient later; lenient only | Ruby's non-strict mode is how most production deployments work; policy abstraction makes both easy |
| Error mode type | `ErrorMode::Strict \| ErrorMode::Lenient(fn(&Error) -> String)` | Separate `strict: bool` + formatter field; enum with Suppress variant | Enum is self-documenting; `fn` pointer in Lenient carries the formatter |
| Error formatter type | `fn(&Error) -> String` (function pointer) | `Box<dyn Fn>` (boxed closure); `Arc<dyn Fn>` (arc'd closure); `Option<fn>` with default | Zero-cost, Copy, Send+Sync, no allocation, no lifetime. Error already carries all context (template name, line, message) — no captured state needed |
| `RenderOptions` generics | No generics, no lifetimes | Lifetime parameter for borrowed formatter; generic over formatter type | Plain struct is simplest; `fn` pointer avoids the need for generics |
| `strict_variables` in RenderOptions | `bool`, default `false` | Not included (leave to caller); `Option<bool>` | Separate from ErrorMode — controls whether lookup produces error vs nil. Default false matches Ruby Liquid |
| `strict_filters` in RenderOptions | `bool`, default `false` | Not included; handled by FallbackFilterRegistry only | Separate from ErrorMode — controls undefined filter behavior. Default false matches Ruby Liquid |
| `LenientObject` location | Moves from `ruby-ext` to production engine | Keep in `ruby-ext` only | Needed in production for `strict_variables: false`; not a conformance-only concern |
| Error message normalization | Stays in `ruby-ext` conformance shim | Move to engine; normalize in both | Ruby-specific message formats ("undefined filter X") are conformance behavior. Production uses raw Error metadata |

## Safety Limits

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| `max_output_bytes` enforcement | `CountingWriter::write()` checks on every write, returns `io::Error` immediately | Check at element boundaries only; advisory check | Hard limit — a single expression writing 100MB is stopped mid-write |
| `io::Error` propagation | Sentinel `ErrorKind`, mapped to liquid error with "output limit exceeded" | Poison flag + check; custom `io::Error` payload with downcast | Simple, no type coupling between io path and liquid error types |
| `max_render_ops` definition | One op per `Renderable::render_to` invocation | One op per template body; one op per expression evaluation | For-loop iterations accumulate naturally (body nodes x iterations); best proxy for pathological input |
| `max_render_ops` enforcement | Checked at element boundaries by executor | Checked inside renderables; checked on every expression | Element boundaries are where the executor already wraps render_to calls |
| `max_assign_bytes` enforcement | Checked at element boundaries by executor; reported from inside assign/capture | Executor measures externally; always-on counter | Only assign_tag and capture_block know their sizes |
| Depth limit | `max_depth` in `RenderOptions` as `Option<usize>` | Hardcoded constant (like Ruby's 100); separate `max_partial_depth`; defer entirely | User-configurable, consistent with other limits; None = unlimited matches the defaults decision |
| Depth model | Combined scope + partial depth (matching Ruby's `base_scope_depth + @scopes.length`) | Partial depth only; separate counters for scope and partial | Matches Ruby semantics; single counter is simpler; protects against both deep nesting and deep recursion |
| Depth error | "nesting too deep" matching Ruby's `StackLevelError` | Generic "limit exceeded"; stack overflow (let it crash) | Consistent with Ruby; explicit error is better than stack overflow |

## Conformance & Ruby Integration

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| `LiveScopeSession` in production | Compile-time gated behind `conformance-harness`; absent in production builds | Always present but unused; runtime flag | Production should compile zero conformance code; `#[cfg]` is the cleanest cut |
| Ruby tag prepass | Built in initial refactor, not deferred | Defer until conformance test requires it; design slot only | Ruby tags are registered but don't work today; architecture should be complete from the start |
| Prepass strategy | Positional extraction of tag regions into opaque placeholder tokens | Semantic string rewriting; parallel AST merge; dynamic grammar rules | Preserves regions without changing Liquid semantics; core parser sees placeholders as opaque renderables |
| Ruby `ResourceLimits` in production | Conformance-only; not in production structs | Always present; production subset | Production has its own limit API; Ruby-compatible counters are conformance overhead |
| Conformance module visibility | `#[doc(hidden)] pub` behind feature gate | Fully public; fully private | Unstable, not part of supported API; only `ruby-ext` consumes it |

## Migration & Execution

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| Migration strategy | One merged change with phased validation on a single branch | Multiple shipped steps; temporary compatibility flag | Keeps the shipped surface coherent while still allowing implementation checkpoints during development |
| `DynamicFilterRuntime` decomposition | Filters → FallbackFilterRegistry; errors/limits/scope → RubyConformancePolicy; data → slim Runtime wrapper | Keep DynamicFilterRuntime as policy impl; extract one piece at a time | Full decomposition in one change; no intermediate states where both old and new paths coexist |

## Implementation Details

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| `RenderPolicy` trait methods | 5 methods: `on_render_op`, `on_render_error`, `on_assign`, `on_scope_push(partial)`, `on_scope_pop` | Fine-grained methods (7+); coarser combined methods (3) | One method per call-site concept. `on_scope_push(partial: bool)` folds depth + partial-entry into one call since they share a call site |
| `ProdPolicy` configuration | Constructed from `RenderOptions` by copying values. Owns counters (ops, assign_bytes, depth) and collected errors `Vec<Error>` | Hold reference to RenderOptions; store values separately | No references/lifetimes. All fields Copy or owned. Policy is a plain struct in registers |
| Depth counter storage | Part of policy state (field on `ProdPolicy`) | Separate register; shared counter via Arc | One register holds all limit state. SandboxedStackFrame propagates one thing and depth comes along. StackFrame shares parent registers so depth is shared automatically |
| Error collection in lenient mode | `render_with_options` returns `Result<RenderOutput>` with output + `Vec<Error>`. Errors appear both inline in output AND in the collected vec | Return `Result<String>` (no collection); collect in separate parameter; defer to v2 | Matches Ruby (inline errors + template.errors). `Vec<Error>` is zero-cost when empty. RenderOutput is a trivial struct |
| `FallbackFilterRegistry` type | Shared `Rc<dyn FallbackFilterResolver>` in the hidden conformance module | `PluginRegistry<Box<dyn ParseFilter>>` (same as main registry); `HashMap<String, fn(...)>` | The resolver is render-local, needs Ruby state, and is propagated through isolated scopes without becoming part of the public production API |
| Prepass mechanism | Register Ruby tags as `ParseBlock`/`ParseTag` in `PluginRegistry` before calling core parser. Opaque renderables delegate to Ruby at render time | String rewriting (extract + replace); parallel AST merge; grammar changes | Uses existing extension mechanism. No string manipulation, no grammar changes, no new hooks in core. `ParseBlock`/`ParseTag` impls live in ruby-ext |

## Thread Safety

| Decision | Choice | Alternatives Considered | Rationale |
|---|---|---|---|
| Threading model | Preserve current: Template is Send+Sync, each render creates own Runtime, single-threaded internally | Support reusable Runtime across renders; support concurrent rendering within single template | Current model is correct for a template engine (parse once, render many); RefCell in Registers is appropriate for sequential tree walk |
