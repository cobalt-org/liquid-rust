# Plan: Run Upstream Shopify Ruby Liquid Tests Against `liquid-rust`

## Goal

Build new infrastructure that allows the real upstream `Shopify/liquid` Ruby test suite to run against this Rust fork, then use that suite as the single source of truth until the Rust-backed implementation passes all upstream Ruby Liquid tests for the pinned upstream revision under test, including tests that inspect Ruby-visible internal APIs and object graphs.

## Decision Summary

- The authoritative compatibility suite is the upstream Ruby test suite from a pinned local checkout of `Shopify/liquid`.
- New infrastructure is required. The existing Rust-side mirrored conformance tests remain useful, but they are not the source of truth.
- Production is supposed to be a pure Rust engine. The Ruby layer is test-only conformance infrastructure behind feature-gated entrypoints.
- The harness should use an in-process Ruby replacement `liquid` gem backed by a Rust native extension, but only as conformance infrastructure, not as the long-term production architecture.
- Because the goal is the full upstream suite unchanged, the harness must reproduce the Ruby-visible internal APIs and object graphs that upstream tests observe, while delegating engine behavior to Rust where possible.
- A subprocess CLI bridge is not the recommended main architecture because too much of the upstream suite depends on object identity, mutable runtime state, registers, warnings, errors, and template lifecycle semantics.
- Success means:
  - the full upstream suite can be executed against the Rust-backed replacement gem
  - failures are attributable and actionable
  - the failure count is driven to zero for the pinned upstream revision

## Objective

Run the upstream `Shopify/liquid` Ruby suite against `liquid-rust` with the real upstream repository checked out locally, then use failures from that suite to drive parser, renderer, filter, tag, runtime, and Ruby-API compatibility work in the Rust fork until the suite passes completely.

## Scope

This plan covers:

- cloning and pinning the upstream `Shopify/liquid` repository
- validating the baseline upstream Ruby environment
- creating a local replacement `liquid` gem that upstream tests load instead of the Ruby implementation
- bridging Ruby test execution into `liquid-rust` through a Rust native extension
- reproducing the Ruby-visible internal APIs and object graphs that the upstream suite inspects
- classifying failures by engine semantics, Ruby API compatibility, and harness gaps
- using the upstream suite as the primary compatibility burn-down signal
- promoting important fixes into fast Rust-native regression tests after they are proven by the upstream suite

This plan does not treat the current Rust-side mirrored conformance tests as authoritative. They remain a secondary regression layer only.

## Working Assumptions

- Upstream checkout path will be `../shopify-liquid`
- `liquid-rust` remains the engine under test
- The upstream Ruby suite should remain unchanged wherever possible
- Acceptable local redirection may include dependency wiring, load-path wiring, or a small preloaded bootstrap shim outside the upstream checkout, as long as upstream test files remain unchanged
- Compatibility is measured against a pinned upstream branch and commit, not an unpinned moving target
- The authoritative baseline includes the pinned Ruby interpreter, Bundler version, and resolved dependency snapshot or container image used to run that pinned upstream revision
- The end goal is full-suite pass for that pinned revision, not partial compatibility
- Full-suite pass includes upstream unit tests that inspect Ruby-visible internal APIs and object graphs, not just rendered output and public entrypoints

## Source of Truth Policy

The upstream Ruby suite is the single source of truth for Ruby Liquid compatibility and conformance behavior.

That means:

- new behavioral work is considered correct only when the upstream Ruby test that exercises it passes against the Rust-backed gem
- when an upstream test asserts Ruby-visible internals, that observed API or object graph is part of the conformance target for this project
- mirrored or translated Rust tests may be added for speed, but they do not replace the upstream result
- if Rust-native tests and upstream Ruby tests disagree on shared Liquid behavior, the upstream Ruby test wins
- production-only features or safety controls may exist after the pinned upstream suite is fully green, but they must not redefine the Ruby-compatible baseline that this harness is validating
- the current mirrored tests under `tests/conformance_ruby` are useful for fast iteration and historical context, but they are secondary

## Success Criteria

### Minimum

- The upstream `Shopify/liquid` repository is cloned locally and pinned to a recorded commit
- The authoritative baseline records the exact Ruby version, Bundler version, and resolved dependency snapshot or container image used for the upstream run
- The upstream suite passes normally against the upstream Ruby implementation
- The upstream suite can be launched against a local Rust-backed replacement `liquid` gem
- A complete test report can be produced for that run

### Target

- The full upstream suite executes against the Rust-backed replacement gem without harness crashes or missing-entrypoint failures
- Ruby-visible internal APIs and object graphs exercised by the pinned upstream suite are exposed by the compatibility layer with test-compatible behavior
- Every failing upstream test is categorized and tracked
- Fixes in `liquid-rust` are validated by rerunning the same upstream Ruby tests

### Finish Line

- The pinned upstream Ruby suite passes completely against the Rust-backed replacement gem, including tests that inspect Ruby-visible internals
- The Rust-native secondary regression layer covers the highest-value fixed behaviors for faster future iteration

## Constraints

- The upstream tests were written against the Ruby gem API, not a Rust crate API
- A direct `cargo test` flow is insufficient
- Many upstream tests depend on in-process Ruby semantics, including:
  - `Liquid::Template`
  - `Liquid::Context`
  - mutable assigns and registers
  - partial loading and caching behavior
  - warning and exception surfaces
  - persistent template state
  - resource limit accounting
  - object hooks and `Drop`-style behavior
- Some upstream tests also assert Ruby-visible internal APIs and object graphs, including parser helpers, parse tree objects, tokenizer surfaces, and class inheritance or registration behavior
- Compatibility issues may come from either:
  - core Liquid semantics in Rust
  - Ruby API shape and lifecycle semantics in the harness
  - Ruby-visible internal object graph shape and class behavior
  - conversion behavior between Ruby objects and Rust values

## Recommended Architecture

Use an in-process replacement gem architecture for the conformance harness from the start.

This is not the long-term production runtime architecture. Production remains a pure Rust engine; the replacement gem and Ruby extension exist only to run the upstream Ruby suite unchanged against feature-gated conformance entrypoints.

### Top-Level Shape

1. Upstream checkout
   - a real pinned local checkout of `Shopify/liquid`
   - used to run the authoritative suite

2. Local replacement `liquid` gem
   - loaded by the upstream suite instead of the upstream Ruby gem
   - preserves Ruby-facing constants, classes, and method names expected by the tests
   - exists only for conformance testing, not for production embedding

3. Rust native extension
   - called in-process from the replacement gem
   - acts as the conformance adapter into the pure-Rust engine
   - owns only conformance-only entrypoints and adapter state; it is not the production runtime surface

4. Secondary Rust regression suite
   - keeps fast coverage for already-understood compatibility fixes
   - never replaces the upstream suite as the final signal

### Why In-Process Instead of a CLI Bridge

A subprocess bridge is not the recommended main path because it is a poor fit for tests that depend on:

- template object identity across parse and render operations
- context and register mutation across nested renders
- partial loading policies
- resource limit state
- warning collection
- exception class behavior
- Ruby object hooks and callback-like behavior

A CLI bridge may still be useful as a short-lived exploratory spike if needed, but it should not be the planned production harness.

### Ruby Side Responsibilities

Within the conformance harness only, the replacement gem should own the Ruby API surface expected by upstream tests, including:

- `Liquid` module entrypoints
- `Liquid::Template`
- `Liquid::Context`
- Ruby-visible internal classes and objects that the suite inspects directly, such as parse-tree, parser, tokenizer, and lookup-facing surfaces where required
- error and warning classes
- file-system and include or render hook integration points
- registration and execution surfaces for Ruby-defined tags, blocks, filters, drops, and related environment overrides
- object coercion and wrapper behavior needed to preserve Ruby-visible semantics
- any compatibility glue needed to keep upstream test code unchanged

### Rust Side Responsibilities

The pure-Rust engine should own the engine behavior, and the Rust extension should expose only the conformance adapter surface into that engine. Engine ownership includes:

- parsing
- rendering
- runtime scope behavior
- variable lookup semantics
- filters, tags, and blocks where ownership can remain in Rust without breaking Ruby-visible compatibility requirements
- partial resolution behavior
- resource limit accounting
- template and context state representation behind Ruby-visible wrappers or proxies where needed in conformance mode

### Bridge Responsibilities

The bridge between Ruby and Rust is conformance-only adapter infrastructure. It should provide:

- stable native handles for long-lived template and context objects
- Ruby-to-Rust value conversion
- Rust-to-Ruby value conversion
- Ruby callback dispatch for filters, tags, blocks, drops, `liquid_method_missing`, `to_liquid`, `to_liquid_value`, and proc or lambda-backed values when the conformance harness requires them
- error and warning translation for conformance runs
- lifecycle management for template and context state
- Ruby-visible wrapper or proxy objects for upstream-observed internals such as template roots, node lists, parse contexts, tokenizers, and lookup nodes when the suite asserts on them
- a clear contract for partial and file-system callbacks

This bridge must not reintroduce generic runtime interposition into production. Dynamic Ruby callback dispatch, Ruby-visible internals, and other adapter behavior belong behind feature-gated conformance entrypoints only.

## Execution Model

The intended harness behavior is:

1. The upstream Ruby suite loads the local replacement `liquid` gem.
2. `Liquid::Template.parse` and related entrypoints resolve inside the replacement gem.
3. The replacement gem delegates engine work to the Rust native extension in-process.
4. Template objects, contexts, and runtime state remain stable across parse and render calls in the same Ruby process.
5. When upstream tests inspect Ruby-visible internals, the replacement gem returns Ruby objects whose shape and behavior match upstream expectations closely enough for the pinned suite.
6. Upstream test assertions observe Ruby-facing behavior through the replacement gem exactly as they would against the upstream implementation.

This execution model applies only to the conformance harness. Production execution continues to use the pure-Rust engine directly.

## Repository Layout Proposal

### In `liquid-rust`

- `SHOPIFY_RUBY_TEST_HARNESS_PLAN.md`
- `harness/ruby-liquid/`
  - local replacement `liquid` gem
  - Ruby compatibility layer
  - native extension build wiring
- `crates/ruby-ext/`
  - Rust extension crate used by the replacement gem
- production crates remain the default workspace members
- conformance-only code is gated behind a dedicated feature such as `conformance-harness`
- root production builds and tests must not require Ruby or build the conformance adapter by default
- future harness documentation
- existing Rust-native compatibility and regression tests

### In parent workspace

- `../shopify-liquid`
  - pinned authoritative upstream checkout

## Delivery Phases

### Phase 1: Upstream Baseline and Pinning

- Clone `Shopify/liquid`
- Record the exact branch and commit under test
- Record the exact Ruby interpreter version required for the baseline run
- Record the exact Bundler version used for the baseline run
- Capture the resolved dependency snapshot used for the baseline run, or record the container image digest if the baseline is containerized
- Verify Ruby and Bundler compatibility for the pinned revision
- Run the upstream suite unchanged against the upstream Ruby implementation
- Capture the canonical baseline command used to run the suite

Deliverables:

- pinned upstream revision reference
- baseline environment manifest covering Ruby version, Bundler version, and resolved dependency snapshot or container image
- reproducible baseline command sequence
- confirmation that the upstream suite is green in its native environment

### Phase 2: Dependency Redirection Strategy

- Choose a single explicit harness activation mechanism that runs before upstream `test/test_helper.rb`
- Use a preloaded bootstrap shim outside the upstream checkout as the default activation mechanism
- Keep upstream test files unchanged
- Allow only external bootstrap, dependency, and load-path wiring needed to redirect `require 'liquid.rb'` and related `liquid/*` loads into the local harness
- Intercept both `require 'liquid.rb'` and `require 'liquid/profiler'` reliably, even though upstream `test/test_helper.rb` prepends the upstream `lib` directory to `$LOAD_PATH` before requiring them
- Add a fail-fast proof that the active `Liquid` implementation came from the local harness rather than the upstream checkout's `lib/liquid.rb`
- Document exactly how to switch the upstream suite between:
  - upstream Ruby implementation
  - local Rust-backed replacement gem

Deliverables:

- agreed harness loading strategy
- documented bootstrap activation command plus proof-of-load check
- reproducible command sequence for running upstream tests against local gem code

### Phase 3: Replacement Gem Skeleton

- Create the local Ruby replacement gem structure
- Mirror the upstream gem's top-level entrypoints, constant layout, and the Ruby-visible internal classes exercised by the pinned suite
- Inventory the Ruby-visible APIs, object graphs, and class relationships that the pinned suite asserts directly
- Define the Ruby-visible API contract that must be satisfied for the suite to boot and for inspected objects to look upstream-compatible
- Establish the native extension boundary

Deliverables:

- documented Ruby API and observed-internal surface to support
- documented wrapper or proxy strategy for Ruby-visible internals that remain backed by Rust
- documented ownership split between Ruby glue and Rust extension

### Phase 4: Minimal In-Process Parse and Render Path

- Make the real upstream `test/test_helper.rb` able to boot against the replacement gem
- Provide a loadable `liquid/profiler` compatibility file so helper boot does not fail during `require`
- Implement `Liquid::Template.parse` keyword compatibility for `line_numbers:`, `error_mode:`, and `environment:`
- Implement `Liquid::Template#parse` compatibility so `Template.new.parse(...)` works
- Implement `Liquid::Registers.new(file_system:, template_factory:)` compatibility
- Implement `Liquid::Context.build(static_environments:, rethrow_errors:, registers:, environment:)` compatibility
- Implement `template.render(context)` compatibility for helper-driven assertions
- Implement the minimum helper-facing Ruby API surface needed for the first upstream test file to execute, including:
  - `Liquid::Environment.default`
  - `Liquid::Environment.build`
  - `Liquid::Registers`
  - `Liquid::Context.build`
  - `Liquid::Template.new`
  - `Liquid::Template.parse`
  - `Liquid::Drop`
  - `Liquid::Tag`
  - `Liquid::Block`
  - `render`
  - `render!`
  - basic errors, warnings, register handling, and environment propagation
- Treat profiler behavior as stub-compatible in this phase; helper boot and first-file execution matter more than real profiling data collection
- Prefer helper boot reachability and API completeness over early semantic completeness

Deliverables:

- first upstream parse-error smoke target, first successful helper-driven render smoke target, `Template#parse` smoke target, and `render!` smoke target executing against the Rust-backed gem
- first end-to-end parse and render path in-process with `line_numbers:`, `error_mode:`, `environment:`, `Template#parse`, `Registers.new(...)`, `Context.build(...)`, and `template.render(context)` compatibility in place
- loadable `liquid/profiler` shim sufficient for helper boot

### Phase 5: Full-Suite Reachability

- Expand the harness until the entire upstream suite can be executed against the Rust-backed gem
- Eliminate harness-level blockers such as missing constants, missing classes, missing methods, or missing callback surfaces
- Add Ruby-visible wrapper or proxy objects for suite-observed internals such as `Template#root`, parse-tree nodes and node lists, `ParseContext`, tokenizer surfaces, variable lookup objects, and related locale or error-state holders
- Ensure Ruby-defined tags, blocks, filters, drops, and environment overrides participate correctly in parse and render flow
- Distinguish tests that fail because of actual compatibility issues from tests that fail because the harness cannot express the expected API yet
- Prove at least one Ruby-defined filter path and one contextual drop path work end-to-end across the Ruby to Rust boundary before broad failure triage begins

Deliverables:

- full upstream suite runnable against the Rust-backed gem
- machine-readable or easily triageable failure inventory

### Phase 6: Failure Accounting and Burn-Down

- Classify every failing upstream test
- Fix the highest-leverage compatibility gaps first
- Re-run the same upstream tests after each fix
- Maintain a live burn-down list tied to exact upstream test names

Deliverables:

- categorized failure tracker
- shrinking failure count
- traceability from test failure to fix

### Phase 7: Secondary Regression Coverage

- After an upstream Ruby test is made to pass, add or update fast Rust-native regression coverage for the same behavior where useful
- Use the Rust-native suite for fast iteration only
- Keep final acceptance tied to the upstream Ruby suite

Deliverables:

- faster secondary regression layer
- reduced cost of repeated compatibility work

### Phase 8: Repeatable Compatibility Workflow

- Standardize the commands for:
  - upstream baseline run
  - Rust-backed harness run
  - failure reporting
  - focused reruns by upstream file or test name
- Prepare the workflow for eventual automation in CI after local stability is achieved

Deliverables:

- stable local workflow
- CI-ready command structure

## Failure Classification

Every failing upstream test should be placed into one primary bucket:

1. Rust semantic mismatch
   - parser behavior
   - runtime lookup behavior
   - filter semantics
   - tag or block behavior
   - whitespace behavior
   - output formatting

2. Ruby API contract mismatch
   - method shape
   - return value shape
   - constant or class layout
   - exception or warning class behavior
   - template or context lifecycle semantics
   - observable internal object graph shape or class identity exposed to Ruby

3. Bridge conversion mismatch
   - Ruby object to Rust value conversion
   - Rust value to Ruby object conversion
   - mutable state reflection across the boundary

4. Partial or file-system integration mismatch
   - include or render lookup behavior
   - callback or hook integration
   - caching or invalidation behavior

5. Harness bug or missing infrastructure
   - missing entrypoint
   - incorrect native handle management
   - unsupported callback surface
   - load-path or dependency wiring problem

The goal of classification is to prevent ambiguous failures. Every failing test should have a clear owner and next action.

## Test Strategy

Use three layers of test execution:

1. Upstream Ruby baseline
   - verifies the pinned upstream checkout and Ruby environment
   - must stay green against the real upstream implementation

2. Upstream Ruby suite against the Rust-backed replacement gem
   - authoritative compatibility signal, including Ruby-visible internal compatibility where the upstream suite inspects it
   - main burn-down metric

3. Native Rust regression tests
   - secondary fast feedback
   - added after upstream proof, not instead of it

## Progress Metrics

Track progress using:

- pinned upstream branch and commit
- total upstream test count for that revision
- tests executable against replacement gem
- tests passing against replacement gem
- tests failing by classification bucket
- tests blocked by missing harness infrastructure
- number of fixes validated upstream and mirrored into Rust-native regression tests

## Risks

- Ruby version or Bundler mismatch may block a clean upstream baseline
- Dependency resolution drift may change the baseline even when the upstream commit stays fixed
- The Ruby gem API surface may be larger than expected
- The Ruby-visible internal surface required by upstream unit tests may be much larger than expected
- Object lifecycle behavior may require richer native handle management than the initial bridge design assumes
- Partial loading and callback surfaces may force more Ruby-side compatibility glue than initially planned
- Ruby callback dispatch across the language boundary may be broader than initially planned
- Error formatting and warning behavior may be highly test-visible
- Full-suite pass may require support for corners of the API that are not currently modeled in `liquid-rust`

## Risk Mitigations

- Pin the upstream revision early and do not chase moving target failures
- Record and preserve the full baseline execution environment, not just the upstream commit
- Get the native environment green before touching the Rust-backed harness
- Choose in-process extension architecture from the beginning
- Separate harness failures from engine failures immediately
- Inventory upstream-observed Ruby-visible internals early and decide which must be native Ruby wrappers versus direct Rust-backed objects
- Prove the harness loaded the replacement gem before trusting any result from a Rust-backed run
- Prefer complete suite reachability early, even with many failures, over polishing a narrow subset first
- Add fast Rust-native regressions only after an upstream test has been made green

## Immediate Next Steps

1. Clone and pin the upstream `Shopify/liquid` repository.
2. Record the exact Ruby version, Bundler version, and dependency snapshot or container image used for the upstream baseline run.
3. Record the exact upstream command used to run its suite.
4. Verify the upstream suite is green in its own environment before introducing any local override.
5. Define the preloaded bootstrap activation mechanism and proof-of-load check so the upstream suite resolves the local replacement `liquid` gem instead of the upstream Ruby implementation, specifically for `require 'liquid.rb'` and `require 'liquid/profiler'`.
6. Write down the minimum helper-facing Ruby API surface needed for the real upstream test helper to boot against the replacement gem, including the `liquid/profiler` shim, `Template.parse` keyword compatibility, and boot-time classes such as `Liquid::Drop`, `Liquid::Tag`, and `Liquid::Block`.
7. Inventory the Ruby-visible internal APIs and object graphs exercised by the pinned upstream suite, including `Template#root`, parse-tree nodes, `ParseContext`, tokenizer surfaces, variable lookup objects, and custom tag or block inheritance behavior.
8. Design the native extension boundary for templates, contexts, values, errors, warnings, partial callbacks, Ruby callback dispatch, and Ruby-visible wrapper or proxy objects for inspected internals.
9. Use `test/integration/document_test.rb`, `VariableTest#test_simple_variable`, `ParsingQuirksTest#test_raise_on_invalid_tag_delimiter`, and `ParsingQuirksTest#test_parsing_css` as the first Phase 4 bootstrap sequence.

## First Target Recommendation

Start with a bootstrap sequence that proves both helper boot and one successful helper-driven render path before broader runtime compatibility work begins.

Phase 4 bootstrap sequence:

1. Parse-error smoke target: `test/integration/document_test.rb`
2. Successful render smoke target: `VariableTest#test_simple_variable` from `test/integration/variable_test.rb`
3. `Template#parse` smoke target: `ParsingQuirksTest#test_raise_on_invalid_tag_delimiter` from `test/integration/parsing_quirks_test.rb`
4. `render!` smoke target: `ParsingQuirksTest#test_parsing_css` from `test/integration/parsing_quirks_test.rb`

This sequence is preferred because it validates:

- real upstream helper boot
- `Template.parse`
- syntax error construction and surfacing
- line-numbered parse errors through helper assertions
- one successful helper-driven parse and render cycle through `assert_template_result`
- `Template.new.parse(...)`
- one successful `render!` path
- basic compatibility of `Liquid::Environment.default`, `Template.parse`, `Registers.new(...)`, `Context.build(...)`, and `template.render(context)`

This bootstrap sequence is intentionally narrower than the full upstream suite. After it is green, the next targets must expand into Ruby-visible internal compatibility that the unit tests inspect.

Phase 5 early parser and runtime targets:

1. `test/integration/parsing_quirks_test.rb`
2. `test/integration/variable_test.rb`
3. `test/integration/output_test.rb`
4. `test/integration/filter_test.rb`

Phase 5 early Ruby-visible internal targets:

1. `test/unit/template_unit_test.rb`
2. `test/unit/parse_context_unit_test.rb`
3. `test/unit/tag_unit_test.rb`
4. `test/integration/block_test.rb`

These files should begin only after compatibility exists for:

- `with_error_modes`
- helper-driven `assert_template_result`
- `Template.new.parse(...)`
- core tags such as `if`, `for`, `assign`, and `liquid`
- stdlib filters used by the selected target files
- `Context.new`
- `Context#add_filters`
- `render!(filters:)`
- `to_liquid`
- `to_liquid_value`
- persistent `template.assigns`

These internal-API targets should begin only after compatibility exists for:

- `Template#root`
- node-list and node-option inspection surfaces used by unit tests
- `ParseContext` helper methods and related parser objects
- `Tokenizer` entrypoints and error behavior
- Ruby-defined `Liquid::Tag` and `Liquid::Block` inheritance and registration behavior
