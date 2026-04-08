# RenderOptions Guide

## Overview

`Template::render` and `Template::render_to` now default to the production rendering behavior:

- pure Rust, with no Ruby dependency in the default build
- missing variables resolve to `nil`
- unknown filters pass the input through unchanged
- render errors are formatted inline and also collected

Use `RenderOptions` when you need stricter behavior or hard safety limits for a specific render call.

## Quick Start

```rust
use liquid::{ErrorMode, ParserBuilder, RenderOptions};

let parser = ParserBuilder::with_stdlib().build().unwrap();
let template = parser.parse("Hello {{ name }}!").unwrap();
let globals = liquid::object!({ "name": "world" });

// Default production behavior
let output = template.render(&globals).unwrap();
assert_eq!(output, "Hello world!");

// Per-render overrides
let result = template
    .render_with_options(
        &globals,
        &RenderOptions {
            max_output_bytes: Some(1_000_000),
            max_render_ops: Some(100_000),
            max_assign_bytes: Some(500_000),
            max_depth: Some(100),
            strict_variables: true,
            strict_filters: true,
            error_mode: ErrorMode::Strict,
        },
    )
    .unwrap();

assert_eq!(result.output, "Hello world!");
assert!(result.errors.is_empty());
```

## Return Types

- `Template::render(&globals) -> Result<String>`
- `Template::render_to(&mut writer, &globals) -> Result<()>`
- `Template::render_with_options(&globals, &options) -> Result<RenderOutput>`
- `Template::render_to_with_options(&mut writer, &globals, &options) -> Result<Vec<Error>>`

`RenderOutput` contains:

- `output: String`
- `errors: Vec<liquid::Error>`

In lenient mode, `output` includes inline formatted errors and `errors` preserves the original error values.

## Defaults

`RenderOptions::default()` means:

- `max_output_bytes = None`
- `max_render_ops = None`
- `max_assign_bytes = None`
- `max_depth = None`
- `strict_variables = false`
- `strict_filters = false`
- `error_mode = ErrorMode::Lenient(default formatter)`

That is the same behavior used by `Template::render` and `Template::render_to`.

## Error Modes

### `ErrorMode::Strict`

Abort on the first render error and return `Err(liquid::Error)`.

```rust
let options = RenderOptions {
    error_mode: ErrorMode::Strict,
    ..RenderOptions::default()
};
```

### `ErrorMode::Lenient(fn(&Error) -> String)`

Continue rendering, write the formatted error inline, and collect the original `Error`.

```rust
let options = RenderOptions {
    error_mode: ErrorMode::Lenient(|error| format!("[liquid-error:{error}]")),
    ..RenderOptions::default()
};
```

## Strictness Flags

### `strict_variables`

- `false` (default): undefined variables resolve to `nil`
- `true`: undefined variables raise an error

### `strict_filters`

- `false` (default): undefined filters return the input unchanged
- `true`: undefined filters raise an error

These flags control whether an error is produced. `error_mode` controls what happens after that error exists.

## Safety Limits

All limits are cumulative across the full render tree. They do not reset at `{% include %}` or `{% render %}` boundaries in the production API.

### `max_output_bytes`

Hard output cap enforced during writes, not only between nodes.

### `max_render_ops`

Caps the total number of rendered nodes across the render tree.

### `max_assign_bytes`

Caps bytes assigned or captured through `{% assign %}` and `{% capture %}`.

### `max_depth`

Caps nested partial/render depth.

## Common Patterns

### User-submitted templates

```rust
let options = RenderOptions {
    max_output_bytes: Some(10_000_000),
    max_render_ops: Some(1_000_000),
    max_assign_bytes: Some(5_000_000),
    max_depth: Some(100),
    strict_variables: false,
    strict_filters: false,
    error_mode: ErrorMode::Lenient(|error| error.to_string()),
};
```

### Trusted internal templates

```rust
let options = RenderOptions {
    strict_variables: true,
    strict_filters: true,
    error_mode: ErrorMode::Strict,
    ..RenderOptions::default()
};
```

### Writer-based rendering

```rust
let mut output = Vec::new();
let errors = template.render_to_with_options(
    &mut output,
    &globals,
    &RenderOptions {
        max_output_bytes: Some(1_000_000),
        ..RenderOptions::default()
    },
)?;
```

## Notes

- The hidden `conformance-harness` path exists only for the Shopify compatibility harness and is not part of the supported production API.
- `liquid` still works with `default-features = false`, so consumers can build a minimal parser with only custom Rust filters, tags, and blocks.
