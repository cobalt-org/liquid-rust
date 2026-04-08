# Tasks: Pure-Rust Production Engine with Ruby-Only Conformance Harness

**Input**: Design documents from `specs/001-prod-conformance-split/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: Required. The feature spec defines independent tests, acceptance scenarios, and measurable outcomes for each user story, so each story includes explicit test tasks.

**Organization**: Tasks are grouped by user story to preserve independent implementation and validation, with setup and foundational work separated out as shared prerequisites.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other tasks in the same phase
- **[Story]**: User story label (`[US1]`, `[US2]`, `[US3]`, `[US4]`)
- Every task includes exact file path(s)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the workspace and harness metadata needed for the split production/conformance design.

- [X] T001 Update workspace `default-members` and root `conformance-harness` feature plumbing in `Cargo.toml`
- [X] T002 [P] Add the `conformance-harness` feature definition to `crates/core/Cargo.toml`
- [X] T003 [P] Propagate the `conformance-harness` feature through `crates/lib/Cargo.toml`
- [X] T004 [P] Enable the `conformance-harness` feature on the `liquid` bridge dependency in `crates/ruby-ext/Cargo.toml`
- [X] T005 [P] Normalize the pinned harness commit and Ruby 3.4.1 baseline in `harness/baseline.yml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create the internal scaffolding that every user story depends on.

**⚠️ CRITICAL**: No user story work should start until this phase is complete.

- [X] T006 Create sealed render-policy module scaffolding in `crates/core/src/runtime/policy.rs` and wire it from `crates/core/src/runtime/mod.rs`
- [X] T007 [P] Create hidden conformance module scaffolding in `crates/core/src/conformance.rs` and gate it from `crates/core/src/lib.rs`
- [X] T008 [P] Introduce register-carried shared handle types for active policy, fallback filters, and strict-variable mode in `crates/core/src/runtime/runtime.rs`
- [X] T009 [P] Prepare root facade exports for the new render API surface in `src/lib.rs`
- [X] T010 Prepare the root template wrapper for configurable render entrypoints in `src/template.rs`

**Checkpoint**: The workspace, feature gates, and module scaffolding are ready for story-by-story implementation.

---

## Phase 3: User Story 1 - Production Rendering Without Ruby (Priority: P1)

**Goal**: Make the default build and default render path pure Rust, Ruby-free, and semantically correct for production consumers.

**Independent Test**: Run root `cargo check` and `cargo test` with Cargo `default-members` and no Ruby installed, then run `cargo test --no-default-features --test minimal_parser`; verify production rendering, stdlib-disabled `ParserBuilder` extensibility, and Rust extension registration still work and conformance-only code does not compile into the default build.

### Tests for User Story 1

- [X] T011 [P] [US1] Add pure-Rust default-build integration coverage in `tests/production_boundary.rs`
- [X] T012 [P] [US1] Add default rendering, Rust extension registration, and stdlib-disabled minimal-parser coverage in `tests/render_api.rs` and `tests/minimal_parser.rs`

### Implementation for User Story 1

- [X] T013 [US1] Implement Ruby-free production `Template::render` and `Template::render_to` entrypoints in `src/template.rs`
- [X] T014 [US1] Shrink the public `Runtime` trait to data-only methods in `crates/core/src/runtime/runtime.rs`
- [X] T015 [US1] Move executor-owned render loop error flow into `crates/core/src/runtime/template.rs`
- [X] T016 [P] [US1] Remove production filter override dispatch from `crates/core/src/parser/filter_chain.rs`
- [X] T017 [P] [US1] Remove policy-hook forwarding from stack frames in `crates/core/src/runtime/stack.rs`
- [X] T018 [US1] Keep production-facing exports Ruby-free while wiring the hidden conformance gate in `src/lib.rs` and `crates/core/src/lib.rs`

**Checkpoint**: The default crate build is Ruby-free and production rendering works through Rust-only APIs.

---

## Phase 4: User Story 2 - Configurable Render Limits and Error Handling (Priority: P2)

**Goal**: Expose configurable render limits and strict/lenient error handling while keeping production semantics cumulative across the full render tree.

**Independent Test**: Render templates with `RenderOptions` under strict and lenient modes, verify hard output limiting, render-op limiting, assign-byte limiting, depth limiting, and collected error reporting.

### Tests for User Story 2

- [X] T019 [P] [US2] Add output, render-op, assign-byte, and depth limit integration coverage in `tests/render_options_limits.rs`
- [X] T020 [P] [US2] Add strict-vs-lenient render API coverage in `tests/render_options_errors.rs`

### Implementation for User Story 2

- [X] T021 [P] [US2] Define `RenderOptions`, `ErrorMode`, and `RenderOutput` in `src/template.rs`
- [X] T022 [US2] Implement `ProdPolicy` counters and collected-error state in `crates/core/src/runtime/policy.rs`
- [X] T023 [US2] Enforce output-byte limits and configurable render output collection in `src/template.rs`
- [X] T024 [US2] Wire executor policy hooks for render ops and render errors in `crates/core/src/runtime/template.rs`
- [X] T025 [P] [US2] Report assign-byte usage from `crates/lib/src/stdlib/tags/assign_tag.rs` and `crates/lib/src/stdlib/blocks/capture_block.rs`
- [X] T026 [P] [US2] Enforce depth tracking and no-reset partial boundaries in `crates/core/src/runtime/stack.rs`, `crates/lib/src/stdlib/tags/include_tag.rs`, and `crates/lib/src/stdlib/tags/render_tag.rs`
- [X] T027 [US2] Implement lenient variable and unknown-filter behavior across nested scopes in `crates/core/src/runtime/runtime.rs`, `crates/core/src/runtime/stack.rs`, and `crates/core/src/parser/filter_chain.rs`

**Checkpoint**: Production renders expose the new configurable API and all four safety limits behave as specified.

---

## Phase 5: User Story 3 - Full Shopify Conformance Via Ruby Harness (Priority: P3)

**Goal**: Preserve the Ruby-backed conformance lane behind a feature gate and make it pass the pinned upstream Shopify Liquid suite.

**Independent Test**: Build with `conformance-harness`, run `make harness-test`, and verify the harness passes against Shopify Liquid commit `a9c85622ddd784078c2eed34b19a351fe57362cf`.

### Tests for User Story 3

- [X] T028 [P] [US3] Add late-filter and normalized-error coverage in `tests/conformance_ruby/filter_test.rs` and `tests/conformance_ruby/error_handling_test.rs`
- [X] T029 [P] [US3] Add isolated render-scope propagation coverage in `tests/conformance_ruby/tags/render_tag_test.rs`

### Implementation for User Story 3

- [X] T030 [US3] Implement hidden conformance parse/render entrypoints in `crates/core/src/conformance.rs` and `src/lib.rs`
- [X] T031 [US3] Implement `RubyConformancePolicy` callbacks and register plumbing in `crates/core/src/runtime/policy.rs` and `crates/core/src/runtime/runtime.rs`
- [X] T032 [US3] Implement feature-gated fallback filter dispatcher lookup in `crates/core/src/parser/filter_chain.rs` and `crates/core/src/parser/registry.rs`
- [X] T033 [US3] Propagate conformance policy, fallback filters, and live-scope state through isolated scopes in `crates/core/src/runtime/stack.rs`
- [X] T034 [US3] Route Ruby template execution through the hidden conformance entrypoints in `crates/ruby-ext/src/template.rs` and `crates/ruby-ext/src/environment.rs`
- [X] T035 [US3] Move Ruby callback, session, and normalized error wiring into `crates/ruby-ext/src/callbacks.rs`, `crates/ruby-ext/src/context.rs`, and `crates/ruby-ext/src/errors.rs`
- [X] T036 [US3] Align executable harness workflow docs with the pinned baseline in `tests/harness/run_shopify_liquid_harness_tests.sh`, `Makefile`, and `harness/README.md`

**Checkpoint**: The conformance harness is feature-gated, hidden from production consumers, and ready to validate against the pinned upstream suite.

---

## Phase 6: User Story 4 - Concurrent Template Rendering (Priority: P4)

**Goal**: Preserve `Template` sharing across threads while keeping runtime and policy state isolated per render.

**Independent Test**: Share a parsed `Template` across threads, render concurrently with different globals and `RenderOptions`, and verify correct isolated output with no cross-thread state leakage.

### Tests for User Story 4

- [X] T037 [P] [US4] Extend shared-template concurrency regression coverage in `tests/multithreading.rs`
- [X] T038 [P] [US4] Add per-render options isolation coverage in `tests/render_options_threads.rs`

### Implementation for User Story 4

- [X] T039 [US4] Ensure each render creates isolated policy state in `src/template.rs` and `crates/core/src/runtime/policy.rs`
- [X] T040 [US4] Remove cross-render shared mutable state from runtime wrappers and register propagation in `crates/core/src/runtime/runtime.rs`, `crates/core/src/runtime/stack.rs`, and `src/template.rs`

**Checkpoint**: Shared templates remain thread-safe and concurrent renders do not share counters, registers, or collected errors.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Finish documentation, validation, and release preparation for the refactor.

- [X] T041 Update public migration and render API docs in `README.md`, `docs/RENDER_OPTIONS_GUIDE.md`, and `CHANGELOG.md`
- [X] T042 [P] Update contributor workflow notes for the split production/conformance lanes in `CONTRIBUTING.md` and `docs/PLAN_DECISIONS.md`
- [X] T043 Run the full validation flow documented in `specs/001-prod-conformance-split/quickstart.md`
- [X] T044 Prepare `1.0.0` release metadata and breaking-change notes in `Cargo.toml`, `crates/core/Cargo.toml`, `crates/lib/Cargo.toml`, and `CHANGELOG.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies
- **Phase 2: Foundational**: Depends on Phase 1 completion and blocks all user story work
- **Phase 3: US1**: Depends on Phase 2 completion; this is the first implementation checkpoint within the single-change rollout
- **Phase 4: US2**: Depends on US1 because the configurable API layers on the pure-Rust production path
- **Phase 5: US3**: Depends on US2 because the conformance bridge builds on the final policy, executor, and render API design
- **Phase 6: US4**: Depends on US2 because concurrency must validate the final per-render options and policy-state model
- **Phase 7: Polish**: Depends on US1, US2, US3, and US4 being complete

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational and has no dependency on later stories
- **US2 (P2)**: Starts after US1 establishes the production-only render path
- **US3 (P3)**: Starts after US2 finalizes policy and render API behavior needed by the Ruby bridge
- **US4 (P4)**: Starts after US2 finalizes per-render state ownership and configurable entrypoints

### Within Each User Story

- Test tasks MUST be written and made to fail before implementation tasks begin
- Public API and data-shape tasks come before executor and integration tasks
- Story-specific validation runs before moving on to the next dependent story

---

## Parallel Opportunities

- **Setup**: T002, T003, T004, and T005 can run in parallel after T001
- **Foundational**: T007, T008, and T009 can run in parallel after T006
- **US1**: T011 and T012 can run together; T016 and T017 can run together after T014 and T015
- **US2**: T019 and T020 can run together; T021 and T022 can run together; T025 and T026 can run together after T022
- **US3**: T028 and T029 can run together; T032 and T033 can run together after T031
- **US4**: T037 and T038 can run together before T039
- **Polish**: T042 can run in parallel with T041 once implementation stabilizes

---

## Parallel Example: User Story 1

```bash
Task: "Add pure-Rust default-build integration coverage in tests/production_boundary.rs"
Task: "Add default rendering and Rust extension registration regression coverage in tests/render_api.rs"

Task: "Remove production filter override dispatch from crates/core/src/parser/filter_chain.rs"
Task: "Remove policy-hook forwarding from stack frames in crates/core/src/runtime/stack.rs"
```

## Parallel Example: User Story 2

```bash
Task: "Add output, render-op, assign-byte, and depth limit integration coverage in tests/render_options_limits.rs"
Task: "Add strict-vs-lenient render API coverage in tests/render_options_errors.rs"

Task: "Report assign-byte usage from crates/lib/src/stdlib/tags/assign_tag.rs and crates/lib/src/stdlib/blocks/capture_block.rs"
Task: "Enforce depth tracking and no-reset partial boundaries in crates/core/src/runtime/stack.rs, crates/lib/src/stdlib/tags/include_tag.rs, and crates/lib/src/stdlib/tags/render_tag.rs"
```

## Parallel Example: User Story 3

```bash
Task: "Add late-filter and normalized-error coverage in tests/conformance_ruby/filter_test.rs and tests/conformance_ruby/error_handling_test.rs"
Task: "Add isolated render-scope propagation coverage in tests/conformance_ruby/tags/render_tag_test.rs"

Task: "Implement feature-gated fallback filter dispatcher lookup in crates/core/src/parser/filter_chain.rs and crates/core/src/parser/registry.rs"
Task: "Propagate conformance policy, fallback filters, and live-scope state through isolated scopes in crates/core/src/runtime/stack.rs"
```

## Parallel Example: User Story 4

```bash
Task: "Extend shared-template concurrency regression coverage in tests/multithreading.rs"
Task: "Add per-render options isolation coverage in tests/render_options_threads.rs"
```

---

## Implementation Strategy

### Internal Checkpoint After US1

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. Validate the Ruby-free production lane with root `cargo check` and `cargo test`
5. Continue on the same branch/PR; this is a validation checkpoint, not a delivery or merge boundary

### Single-Change Execution Order

1. Implement Setup + Foundational to establish the feature-gated architecture
2. Implement US1 to restore a pure-Rust production engine
3. Implement US2 to expose configurable limits and default lenient behavior
4. Implement US3 to reattach the Ruby conformance bridge behind the feature gate
5. Implement US4 to prove concurrent rendering still isolates per-render state
6. Finish with Phase 7 documentation, validation, and release prep
7. Merge or release only after the full refactor and final validation flow are complete

### Parallel Team Strategy

1. One developer handles workspace and foundational scaffolding
2. After US1 stabilizes, split US2 safety-limit work and US3 bridge work across separate owners
3. Use US4 as a late-stage verification slice once the final per-render policy model is in place

---

## Notes

- Every task line follows the required checklist format
- `[P]` is only used for tasks that can be done on separate files without blocking on unfinished work in the same phase
- User story labels are only used on story-specific tasks
- The default execution order is US1 → US2 → US3 → US4 because these stories share the same runtime and parser internals
