# Key Decisions

This crate line makes a few intentional compatibility and architecture choices.

## Filter Resolution

Unknown filters are resolved at render time.

That allows a parsed template to be rendered with filter sets that are only known at runtime.

Known filters are still compiled and validated during parse.

That keeps parse-time argument validation and avoids reparsing registered filters on every render.

In practice, the behavior is:

- known filter with invalid arguments: parse error
- unknown filter with valid syntax: parse succeeds
- unknown filter that is still unresolved during render: render error

## Template Reuse

Compiled templates are reused during render.

The native bridge keeps the compiled template object in the handle and renders that directly instead of reparsing the source string on each call.

This keeps render behavior closer to long-lived template objects and avoids unnecessary parse work.

## Runtime Extension Point

Filter execution now has a runtime hook.

Core rendering can fall back to parser-registered filters, while integrations can override runtime resolution for filters that are supplied dynamically.

This keeps the parser model simple while still allowing runtime-specific behavior when needed.

## Scope Of Change

This is a new crate line with breaking changes.

The goal is not to preserve old parse-time behavior for every caller. The goal is to provide a cleaner model for runtime-provided filters while keeping parser-known filters efficient and validated early.
