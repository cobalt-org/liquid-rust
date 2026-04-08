<!--
  Sync Impact Report
  ==================
  Version change: (none) → 1.0.0
  Bump rationale: MAJOR — initial constitution ratification

  Modified principles: N/A (first version)

  Added sections:
    - Core Principles (5 principles)
    - Technical Constraints
    - Development Workflow
    - Governance

  Removed sections: N/A

  Templates requiring updates:
    - .specify/templates/plan-template.md        ✅ compatible (Constitution Check section is generic)
    - .specify/templates/spec-template.md         ✅ compatible (no principle-specific references)
    - .specify/templates/tasks-template.md        ✅ compatible (no principle-specific references)

  Follow-up TODOs: none
-->

# liquid-rust Constitution

## Core Principles

### I. Conformance First

Incompatibilities with [shopify/liquid](https://github.com/Shopify/liquid)
in strict mode are bugs, not design choices.

- All tags, filters, and output behaviors MUST match the reference
  Ruby implementation unless a deviation is explicitly documented
  and tracked under the `shopify-compatibility` label.
- When conformance and ergonomics conflict, conformance wins.
- New features MUST NOT break existing conformant behavior.

### II. Extensibility

Liquid embraces [language variants](https://shopify.github.io/liquid/basics/variations/)
for different domains. liquid-rust MUST follow that spirit.

- Custom filters, tags, and blocks MUST be registrable through
  the public `ParserBuilder` API.
- The stdlib is optional (`default = ["stdlib"]`); consumers MUST
  be able to build a minimal parser with zero built-in filters.
- Extension points MUST be documented with examples.

### III. Crate Modularity

The workspace is organized into focused crates. Each crate MUST
have a clear, single purpose and MUST compile independently.

- `liquid-core`: value types, traits, and parsing primitives.
- `liquid-derive`: procedural macros for filter/tag authoring.
- `liquid-lib`: standard library of filters, tags, and blocks.
- `liquid` (root): public facade re-exporting the above.
- New crates MUST justify their existence; do not create
  organizational-only crates with no standalone value.

### IV. Performance Discipline

Optimize within the bounds set by Principle I (Conformance).

- Benchmarks (`benches/`) MUST be maintained and run before
  merging performance-sensitive changes.
- Regressions require explicit justification in the PR description.
- Prefer zero-copy and borrowed data where the Liquid semantics
  allow it; avoid unnecessary allocations in hot paths.

### V. Code Quality

- The workspace `[workspace.lints]` section is the single source
  of truth for lint policy. All clippy and rustc warnings listed
  there MUST remain enabled.
- MSRV (Minimum Supported Rust Version) is declared in
  `Cargo.toml` (`rust-version`). Bumping MSRV is a deliberate,
  documented decision — not a side-effect of a dependency update.
- Public API changes MUST follow semver strictly; breaking changes
  require a major version bump.

## Technical Constraints

- **Language**: Rust, edition 2021.
- **MSRV**: 1.83.0 (update deliberately, document in changelog).
- **License**: Dual-licensed MIT OR Apache-2.0. All contributions
  MUST be compatible with this dual license.
- **Dependencies**: Minimize external dependencies. New deps MUST
  justify their addition and SHOULD prefer well-maintained,
  no-unsafe crates where possible.
- **Platforms**: The library MUST compile on all tier-1 Rust
  targets. Platform-specific code MUST be gated behind cfg attrs.

## Development Workflow

- **Testing**: `cargo test --workspace` MUST pass before merge.
  Tests SHOULD cover both conformance (against shopify/liquid
  behavior) and edge cases specific to the Rust implementation.
- **Linting**: `cargo clippy --workspace` with workspace lints
  MUST produce zero warnings.
- **Formatting**: `cargo fmt --check` MUST pass.
- **Benchmarks**: Run `cargo bench` for changes touching parsing,
  rendering, or value conversion paths. Report results in PR.
- **Releases**: Use `cargo-release` workflow. Pre-release
  replacements in `CHANGELOG.md` are automated via
  `[package.metadata.release]`.
- **Commit discipline**: Prefer focused, single-purpose commits.
  Each commit SHOULD compile and pass tests independently.

## Governance

This constitution is the highest-authority document for the
liquid-rust project. When practices conflict with these
principles, the constitution prevails.

- **Amendments**: Changes to this constitution MUST be proposed
  via PR, reviewed by at least one maintainer, and include a
  migration plan for any affected code or workflows.
- **Versioning**: The constitution follows semantic versioning.
  MAJOR for principle removals or redefinitions, MINOR for new
  principles or material expansions, PATCH for clarifications.
- **Compliance**: All PRs and code reviews SHOULD verify
  alignment with these principles. Deviations MUST be justified
  in the PR description.

**Version**: 1.0.0 | **Ratified**: 2026-04-04 | **Last Amended**: 2026-04-04
