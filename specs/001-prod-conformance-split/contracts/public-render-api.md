# Contract: Public Render API

## Scope

Stable consumer-facing contract for the `liquid` crate for the `1.0.0` breaking-change release targeted by this feature.

## Entry Points

| API | Contract |
|---|---|
| `Template::render(&dyn ObjectView) -> Result<String>` | Uses `RenderOptions::default()` semantics: default lenient rendering, Ruby-compatible undefined variable/filter handling, and no Ruby dependency in production builds. |
| `Template::render_to(&mut dyn Write, &dyn ObjectView) -> Result<()>` | Same semantics as `render`, but writes to a caller-provided sink. |
| `Template::render_with_options(&dyn ObjectView, &RenderOptions) -> Result<RenderOutput>` | Advanced entrypoint that exposes limits, strictness, error mode, rendered output, and collected errors. |
| `Template::render_to_with_options(&mut dyn Write, &dyn ObjectView, &RenderOptions) -> Result<Vec<Error>>` | Writer variant of configurable rendering; returns only collected errors because output is already written. |
| `ParserBuilder` registration APIs | Remain the stable production extension surface for Rust filters, tags, blocks, and partials. |

## Type Contract

### `RenderOptions`

- Fields: `max_output_bytes`, `max_render_ops`, `max_assign_bytes`, `max_depth`, `strict_variables`, `strict_filters`, `error_mode`
- All limit fields are `Option<usize>` where `None` means unlimited.
- Default values:
  - `max_output_bytes = None`
  - `max_render_ops = None`
  - `max_assign_bytes = None`
  - `max_depth = None`
  - `strict_variables = false`
  - `strict_filters = false`
  - `error_mode = ErrorMode::Lenient(default formatter)`

### `ErrorMode`

- `Strict`: abort on the first render error.
- `Lenient(fn(&Error) -> String)`: format the error inline and continue.

### `RenderOutput`

- `output: String`
- `errors: Vec<Error>`
- In lenient mode, `output` includes inline formatted errors and `errors` preserves the original error values.

## Behavioral Guarantees

- Default production builds and default feature sets must compile without Ruby, `magnus`, or conformance-only code.
- `liquid` must remain usable with `default-features = false`, allowing consumers to construct a minimal `ParserBuilder`-based parser with zero built-in stdlib filters.
- Production output semantics target Shopify Liquid commit `a9c85622ddd784078c2eed34b19a351fe57362cf`.
- `strict_variables: false` resolves missing variables to `nil` across the full render tree, including nested and isolated scopes.
- `strict_filters: false` passes input through unchanged when no production filter resolution path succeeds.
- `max_output_bytes` is enforced mid-write, not only at element boundaries.
- `max_render_ops`, `max_assign_bytes`, and `max_depth` are cumulative across the entire render tree and do not reset at partial boundaries.
- Lenient mode must both inline formatted errors and preserve the original `Error` values for programmatic access.
- `Template` remains safe to share across threads; each render owns isolated runtime, register, and policy state.

## Unsupported or Removed Surface

- `Template::render_to_runtime` is no longer part of the supported public production API.
- The production engine does not expose a generic runtime callback/interposition API for filters, resource limits, or render-error interception.
- Ruby-specific error message normalization is not part of the production API contract.
