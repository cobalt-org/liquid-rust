# Ruby Harness / Shopify Liquid Parity Handoff

## Scope

This work is about making the Rust-backed Liquid implementation match Shopify Ruby Liquid behavior closely enough that the Ruby harness passes upstream `template_test.rb`, while keeping the real production behavior in Rust.

Important constraint:

- Production will be Rust-only.
- The Ruby harness is only for compatibility validation.
- If behavior must be implemented for parity, prefer implementing it in Rust, not as a Ruby-only fake, unless the Ruby layer is only adapting harness APIs.

## Current Repo State

- Repo root: `.`
- Branch: `codex/shopify-ruby-harness-mvp`
- Refresh state with:

```bash
git status --short
git log --oneline -10
```

## What Is Already Fixed

### Exception renderer parity

The Rust render path now supports Ruby-Liquid-style inline exception replacement and continuation instead of truncating output after the first render error.

Behavior currently restored:

- `A{{ 1 | divided_by: 0 }}B` with default renderer keeps rendering suffix output.
- Custom `exception_renderer` returning a string replaces inline and continues.
- `exception_renderer: ->(e) { raise e }` re-raises the original Liquid exception.
- Nested scopes now continue correctly too, instead of only the top-level template recovering.

Main files involved:

- `crates/core/src/runtime/template.rs`
- `crates/core/src/runtime/runtime.rs`
- `crates/core/src/runtime/stack.rs`
- `crates/ruby-ext/src/template.rs`
- `harness/ruby-liquid/lib/liquid/template.rb`

### Assign / proc / strict lookup parity

The harness now preserves several Ruby Liquid semantics that regressed during the MVP:

- `instance_assigns` persist correctly across parses and renders.
- one-off custom assigns do not leak into later renders.
- persistent assigns override `instance_assigns`.
- zero-arity proc assigns are memoized correctly.
- one-arity proc assigns receive `Context`.
- strict variable handling works for normal values, drops, and proc-backed lookups.
- present `nil` values do not raise under `strict_variables`.
- arbitrary user exceptions are no longer reclassified just because their message contains `undefined variable`.

Main files involved:

- `crates/ruby-ext/src/context.rs`
- `crates/ruby-ext/src/lib.rs`
- `crates/ruby-ext/src/values.rs`
- `harness/ruby-liquid/lib/liquid/context.rb`
- `harness/ruby-liquid/lib/liquid/template.rb`

## Current Upstream Test Status

Last run:

```bash
RBENV_VERSION=3.4.1 ruby \
  -Itest \
  -r ../liquid-rust/harness/bootstrap.rb \
  test/integration/template_test.rb

Current result:

- `37 runs`
- `60 assertions`
- `5 failures`
- `8 errors`

Remaining failures/errors:

1. `test_default_resource_limits_unaffected_by_render_with_context`
   - missing `ResourceLimits#assign_score`
2. `test_resource_limits_hash_in_template_gets_updated_even_if_no_limits_are_set`
   - missing `ResourceLimits#assign_score`
3. `test_cumulative_render_score_limit_raises_on_render_bang`
   - missing `ResourceLimits#cumulative_render_score_limit=`
4. `test_resource_limits_render_length`
   - render length limit not enforced
5. `test_resource_limits_render_score`
   - render score limit not enforced
6. `test_resource_limits_aborts_rendering_after_first_error`
   - limit handling does not abort render
7. `test_render_length_persists_between_blocks`
   - render length accounting not persisting across blocks
8. `test_render_length_uses_number_of_bytes_not_characters`
   - byte-length accounting missing
9. `test_cumulative_render_score_tracks_across_partials_without_limit`
   - partial lookup for `render` failing
10. `test_cumulative_render_score_limit_across_render_tags`
   - partial lookup for `render` failing
11. `test_cumulative_assign_score_limit_across_include_tags`
   - partial lookup for `include` failing
12. `test_using_range_literal_works_as_expected`
   - parser rejects `(x..y)` range literal
13. `test_raises_error_with_invalid_utf8`
   - `TemplateEncodingError` missing / wrong exception path

## Recommended Order To Continue

This is the order I would use in a new thread.

### 1. Resource limits cluster

This is the biggest remaining bucket and likely unlocks multiple tests at once.

Start with:

```bash
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/ruby \
  -Itest \
  -r ../liquid-rust/harness/bootstrap.rb \
  test/integration/template_test.rb --name test_resource_limits_hash_in_template_gets_updated_even_if_no_limits_are_set
```

Then:

```bash
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/ruby \
  -Itest \
  -r ../liquid-rust/harness/bootstrap.rb \
  test/integration/template_test.rb --name test_resource_limits_render_length
```

Things to inspect first:

- `harness/ruby-liquid/lib/liquid/resource_limits.rb`
- `crates/ruby-ext/src/template.rs`
- any Rust runtime/resource-limit code paths already present in core

Likely missing work:

- expose assign/render score fields and setters in the harness API
- wire runtime counters back into `Liquid::ResourceLimits`
- enforce render length by bytes, not characters
- preserve counters across nested blocks / partials

### 2. Partial resolution for `render` / `include`

Once resource limits are moving, fix the partial lookup failures because they block all cumulative partial-related tests.

Start with:

```bash
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/ruby \
  -Itest \
  -r ../liquid-rust/harness/bootstrap.rb \
  test/integration/template_test.rb --name test_cumulative_render_score_tracks_across_partials_without_limit
```

Look at:

- `harness/ruby-liquid/lib/liquid/environment.rb`
- `harness/ruby-liquid/lib/liquid/file_system.rb`
- `crates/ruby-ext/src/template.rs`

Current symptom:

- `render "loop"` is resolving as `loop.liquid` and then failing with `Partial does not exist`
- `include "assign_partial"` also fails even with a stub file system

### 3. Range literal parsing

This is narrow and isolated after the larger API/runtime gaps.

Start with:

```bash
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/ruby \
  -Itest \
  -r ../liquid-rust/harness/bootstrap.rb \
  test/integration/template_test.rb --name test_using_range_literal_works_as_expected
```

Current parse failure:

- `Liquid syntax error (line 1): unexpected Range; expected FilterChain`

This is likely in Rust parser grammar / expression parsing, not in the Ruby harness.

### 4. Invalid UTF-8 exception parity

Start with:

```bash
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/ruby \
  -Itest \
  -r ../liquid-rust/harness/bootstrap.rb \
  test/integration/template_test.rb --name test_raises_error_with_invalid_utf8
```

Current issue:

- `TemplateEncodingError` constant is missing or not raised

Likely fix area:

- `harness/ruby-liquid/lib/liquid/errors.rb`
- parse error mapping in `harness/ruby-liquid/lib/liquid/template.rb`

## Commands That Matter

### Rebuild the Ruby extension after changing Rust or harness glue

Run from:

```bash
./harness/ruby-liquid
```

Command:

```bash
unset RUSTC_WRAPPER; \
RB_SYS_CARGO_TARGET_DIR=/tmp/liquid-ruby-ext-target \
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/bundle exec rake compile
```

### Run one targeted upstream test

Run from:

```bash
../shopify-liquid
```

Command pattern:

```bash
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/ruby \
  -Itest \
  -r ../liquid-rust/harness/bootstrap.rb \
  test/integration/template_test.rb --name TEST_NAME
```

### Run the full upstream template file

```bash
RBENV_VERSION=3.4.1 /Users/ahmed/.rbenv/shims/ruby \
  -Itest \
  -r ../liquid-rust/harness/bootstrap.rb \
  test/integration/template_test.rb
```

## Important Behavior Constraints

These were easy to regress while fixing other issues.

### Do not break these again

- Inline exception replacement must continue rendering after the failing node.
- Nested scopes must forward render-error recovery.
- Re-raising from `exception_renderer` must propagate the original Liquid exception.
- Custom one-off assigns must not leak into later renders.
- Persistent assigns must override `instance_assigns`.
- Zero-arity proc assigns must memoize once.
- One-arity proc assigns must receive `Context`.
- `strict_variables` must distinguish missing from present-`nil`.
- Do not reclassify arbitrary app exceptions just because the message contains words like `undefined variable`.

### Useful smoke checks

These are fast and catch real regressions:

```ruby
Template.parse("{{ 1 | divided_by: 0 }}").render({}, exception_renderer: ->(_) { "ERR" })
# expected: "ERR" or inline "AERRB" when embedded in surrounding text
```

```ruby
t = Liquid::Template.parse("{{ x }}")
t.render!({ "x" => nil }, strict_variables: true)
# expected: ""
```

```ruby
t = Liquid::Template.parse("{{ x }}")
t.render!({ "x" => ->(ctx) { ctx["missing"] } }, strict_variables: true)
# expected: raise Liquid::UndefinedVariable
```

```ruby
mod = Module.new do
  def boom(v)
    raise "undefined variable from app"
  end
end
Liquid::Template.parse("{{ x | boom }}").render!({ "x" => 1 }, filters: [mod])
# expected: generic Liquid::Error, not Liquid::UndefinedVariable
```

## Suggested First Prompt For A New Thread

If starting a fresh AI thread, paste something like:

> Read `liquid-rust/AI_THREAD_HANDOFF.md` first. We are working on Shopify Ruby Liquid compatibility for the Rust-backed harness. Start by running the targeted resource limit tests listed there, inspect the relevant harness and Rust files, and fix the remaining `template_test.rb` failures without regressing the already-fixed exception renderer, assign precedence, proc memoization, or strict variable behavior.
> Read `AI_THREAD_HANDOFF.md` first. We are working on Shopify Ruby Liquid compatibility for the Rust-backed harness. Start by running the targeted resource limit tests listed there, inspect the relevant harness and Rust files, and fix the remaining `template_test.rb` failures without regressing the already-fixed exception renderer, assign precedence, proc memoization, or strict variable behavior. Refresh branch/head context with `git status --short` and `git log --oneline -10`.
