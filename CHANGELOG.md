# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.23.0] - 2021-08-24

### Breaking Changes

- Upgraded from `anymap` to `anymap2` which is slightly better maintained (and removed it from the API)

## [0.22.0] - 2021-02-27

### Breaking Changes

For the most part, only plugin authors should be impacted by these changes.

- `core::runtime` went through significant changes
  - `Renderable::render_to` now takes `&dyn Runtime` instead of `&Runtime<'_>`
  - Adding a new stack frame is now a `StackFrame::new` instead of `Runtime.run_in_scope`
    - This opens up taking references to layers lower in the stack.
    - `runtime.` to access stack functions instead of `runtime.stack_mut()`
- `InterruptState` is now `InterruptRegister` and accessed via `runtime.registers()`
  - Functions were renamed while at it.
- `core::model` has been flattened
- `derive(ValueView)` now requires being used with `impl ObjectView`
- `liquid-core` users now need to opt-in to the `derive` feature for derive macros

### Features

API
- Allow `#[derive(liquid_core::ObjectView, liquid_core::ValueView)]` (previously only worked from `liquid`, making it unusable for the `lib` crate)

### Fixes

- Remove `serde` requirement for `derive(ValueView)`, making it work with more types (like `field: &dyn ValueView`).

### Performance

- Reduce allocations for for-loop variables
- Reduce overhead from `derive(ValueView)` generating `to_value`

Benchmarks:
- Baseline was Liquid 0.21.5
- Variability tended to be high when in the low `us` range
- For the most part, this release brings us in line with Tera's performance (when we weren't already faster).
  - This is with correcting for [a bug in Tera's benchmarks](https://github.com/Keats/tera/issues/606)
- Something is off about `bench_big_loop_big_object/render`, it was hardly
  impacted by the changes and yet that should have been case that greatly
  improved.  [Further investigation is needed](https://github.com/cobalt-org/liquid-rust/issues/434).
```
handlebars_bench_template/parse/handlebars
                        time:   [24.068 us 24.265 us 24.561 us]
                        change: [-19.750% -17.343% -14.831%] (p = 0.00 < 0.05)
                        Performance has improved.
handlebars_bench_template/parse/liquid
                        time:   [19.382 us 19.438 us 19.503 us]
                        change: [-1.7723% -0.6484% +0.3359%] (p = 0.25 > 0.05)
                        No change in performance detected.
handlebars_bench_template/render/handlebars
                        time:   [17.542 us 17.644 us 17.768 us]
                        change: [+0.2705% +0.9005% +1.5616%] (p = 0.01 < 0.05)
                        Change within noise threshold.
handlebars_bench_template/render/liquid
                        time:   [9.0979 us 9.1362 us 9.1840 us]
                        change: [-29.397% -28.804% -28.079%] (p = 0.00 < 0.05)
                        Performance has improved.

handlebars_bench_large_loop/render/handlebars
                        time:   [2.1674 ms 2.1748 ms 2.1828 ms]
                        change: [-1.6970% -0.9970% -0.3973%] (p = 0.00 < 0.05)
                        Change within noise threshold.
handlebars_bench_large_loop/render/liquid
                        time:   [971.57 us 1.0048 ms 1.0330 ms]
                        change: [-25.328% -22.623% -20.133%] (p = 0.00 < 0.05)
                        Performance has improved.

liquid_bench_fixtures/parse/Hello World
                        time:   [1.5103 us 1.5228 us 1.5364 us]
                        change: [+0.9648% +2.0790% +3.2025%] (p = 0.00 < 0.05)
                        Change within noise threshold.
liquid_bench_fixtures/render/Hello World
                        time:   [325.45 ns 327.42 ns 329.17 ns]
                        change: [+66.006% +66.928% +67.840%] (p = 0.00 < 0.05)
                        Performance has regressed.

bench_big_loop_big_object/render/tera
                        time:   [9.7588 us 10.082 us 10.483 us]
                        change: [+16.670% +19.302% +22.469%] (p = 0.00 < 0.05)
                        Performance has regressed.
bench_big_loop_big_object/render/liquid
                        time:   [243.01 us 244.43 us 245.82 us]
                        change: [-3.1405% -2.6344% -2.1172%] (p = 0.00 < 0.05)
                        Performance has improved.

bench_big_table/render/tera
                        time:   [3.6955 ms 3.7093 ms 3.7226 ms]
                        change: [-14.427% -12.151% -9.8055%] (p = 0.00 < 0.05)
                        Performance has improved.
bench_big_table/render/liquid
                        time:   [5.4149 ms 5.4978 ms 5.5944 ms]
                        change: [-56.570% -55.904% -55.143%] (p = 0.00 < 0.05)
                        Performance has improved.

bench_teams/render/tera time:   [8.9949 us 9.0961 us 9.2245 us]
                        change: [+11.039% +14.026% +16.983%] (p = 0.00 < 0.05)
                        Performance has regressed.
bench_teams/render/liquid
                        time:   [9.0989 us 9.1398 us 9.1854 us]
                        change: [-30.075% -29.750% -29.403%] (p = 0.00 < 0.05)
                        Performance has improved.

bench_parsing_basic_template/render/tera
                        time:   [29.527 us 29.732 us 29.979 us]
                        change: [-0.2179% +0.8213% +2.0881%] (p = 0.16 > 0.05)
                        No change in performance detected.
bench_parsing_basic_template/render/liquid
                        time:   [16.676 us 16.724 us 16.772 us]
                        change: [-1.2453% -0.7338% -0.3215%] (p = 0.00 < 0.05)
                        Change within noise threshold.

bench_rendering_only_variable/render/tera
                        time:   [1.4250 us 1.4302 us 1.4351 us]
                        change: [-14.768% -12.240% -9.6021%] (p = 0.00 < 0.05)
                        Performance has improved.
bench_rendering_only_variable/render/liquid
                        time:   [851.04 ns 859.36 ns 867.64 ns]
                        change: [+26.841% +28.148% +29.436%] (p = 0.00 < 0.05)
                        Performance has regressed.

bench_rendering_basic_templates/render/tera
                        time:   [7.9776 us 8.2825 us 8.5379 us]
                        change: [+8.1702% +11.263% +14.601%] (p = 0.00 < 0.05)
                        Performance has regressed.
bench_rendering_basic_templates/render/liquid
                        time:   [3.9680 us 3.9832 us 3.9992 us]
                        change: [+15.551% +16.062% +16.660%] (p = 0.00 < 0.05)
                        Performance has regressed.

bench_huge_loop/render/tera
                        time:   [850.76 us 857.75 us 865.69 us]
                        change: [+0.4801% +0.9708% +1.5712%] (p = 0.00 < 0.05)
                        Change within noise threshold.
bench_huge_loop/render/liquid
                        time:   [803.23 us 808.50 us 814.27 us]
                        change: [-32.892% -32.340% -31.777%] (p = 0.00 < 0.05)
                        Performance has improved.

bench_access_deep_object/render/tera
                        time:   [5.5409 us 5.5652 us 5.5925 us]
                        change: [-3.5414% -2.1804% -1.0822%] (p = 0.00 < 0.05)
                        Performance has improved.
bench_access_deep_object/render/liquid
                        time:   [4.2440 us 4.2770 us 4.3158 us]
                        change: [-43.376% -42.898% -42.391%] (p = 0.00 < 0.05)
                        Performance has improved.

bench_access_deep_object_with_literal/render/tera
                        time:   [7.7474 us 7.7802 us 7.8141 us]
                        change: [+0.9487% +1.4115% +1.9242%] (p = 0.00 < 0.05)
                        Change within noise threshold.
bench_access_deep_object_with_literal/render/liquid
                        time:   [6.2592 us 6.2853 us 6.3135 us]
                        change: [-48.596% -47.174% -45.635%] (p = 0.00 < 0.05)
                        Performance has improved.
```

## [0.21.5] - 2021-02-03

### Features

- `{% include %}`: Support for variable passing for  (#310 closed by #424)
- `{% forloop %}`: Support for `parentloop` variable (#271 closed by #425)

## [0.21.4] - 2020-08-27

### Highlights

- Conformance: support looping over `nil` (Fixes #294)

## 0.21.3 - 2020-08-03

### Highlights

- Fix date serialization

## 0.21.2 - 2020-07-15

### Highlights

- Fix compilation error in lib "extras"

## 0.21.1 - 2020-07-15

### Highlights

- `where` filter is now being included when stdlib is requested.

## 0.21.0 - 2020-07-09

### Breaking Changes

- Switched from `ScalarCow` using `i32` to `i64`.

### Highlights

API

- Switched from `ScalarCow` using `i32` to `i64`.
- Added `from_value(&dyn ValueView)` to complement `to_value(...) -> Vaue`
- `ValueView` support was added to all integer types

## 0.20.2 - 2020-06-12

#### Bug fixes

- Don't crash on bad date formats (see #409)

## 0.20.1 - 2020-06-08

### Highlights

#### Conformance improvements

- Support `split` on `nil` (see #403)

#### Bug fixes

- Fix overflow in truncate (see #402)
- Don't panic on divide-by-zero (see #404)

## 0.20.0 - 2020-03-12

This release resolves several planned breaking changes we've been holding off on.  This doesn't make us ready for 1.0 yet but this closes the gap significantly.

### Highlights

#### Conformance improvements

We're striving to match the liquid-ruby's behavior and this release gets us closer:
- `where` filter implemented by or17191
- Improvements to `sort`, `sort_natural`, `compact`, and other filters by or17191
- Improvements to `include`s conformance.  Before, it was a weird hybrid of jekyll and stdlib styles.
- Support for `{{ var.size }}`
- Improved equality of values

In addition, we've made it more clear what filters, tags, and blocks are a part of core liquid, Jekyll's extensions, Shopify's extensions, or our own extensions.

#### Improved API stability for `liquid`

The `liquid` crate has been stripped down to what is needed for parsing and rendering a template.
- `liquid_core` was created as a convenience for plugin authors.
- `liquid_lib` has all plugins so `liquid` can focus on providing the `stdlib`
  while non-`stdlib` plugins can more easily evolve.

#### `render` can accept Rust-native types

Previously, you had to construct a `liquid::value::Object` (a newtype for a `HashMap`) to pass to `render`.  Now, you can create a `struct` that implements `ObjectView` and `ValueView` instead and pass it in:

```rust
#[derive(liquid::ObjectView, liquid::ValueView, serde::Serialize, serde::Deserialize, Debug)]
struct Data {
    foo: i32,
    bar: String,
}

let data = Data::default();
let template = todo!();
let s = template.render(&data)?;
```

In addition to the ergonomic improvements, this can help squeeze out the most performance:
* Can reuse borrowed data rather than having to switch everything to an owned type.
* Avoid allocating for the `HashMap` entries.

These improvements will be in the caller of `liquid` and don't show up in our benchmarks.

#### Other `render` ergonomic improvements

There multiple convenient ways to construct your `data`, depending on your application:
```rust
let template = todo!();

// `Object` is a newtype for `HashMap` and has a similar interface.
let object = liquid::Object::new();
let s = template.render(&object)?;

let object = liquid::object!({
    "foo" => 0,
    "bar" => "Hello World",
});
let s = template.render(&object)?;

// Requires your struct implements `serde::Serialize`
let data = todo!();
let object = liquid::to_object(&data)?;
let s = template.render(&object)?;

// Using the aforementioned derive.
let data = Data::default();
let s = template.render(&data)?;
```

#### String Optimizations

A core data type in liquid is an `Object`, a mapping of strings to `Value`s. Strings used as keys within a template engine are:
* Immutable, not needing separate `size` and `capacity` fields of a `String`. `Box<str>` is more appropriate.
* Generally short, gaining a lot from small-string optimizations
* Depending on the application, `'static`.  Something like a `Cow<'static, str>`. Even better if it can preserve `'static` getting a reference and going back to an owned value.

Combining these together gives us the new `kstring` crate.  Some quick benchmarking suggests
- Equality is faster than `String` (as a gauge of access time).
- Cloning takes 1/5 the time when using `'static` or small string optimization.

### Details

#### Breaking Changes

* String types have been switched to `kstring` types for small string and `'static` optimizations.
* Plugins (tags, filters, and blocks)
  * Reflection traits are no longer a super trait but instead a getter is used.
  * Filter API changed to accept a `&dyn ValueView`
  * `liquid` is stripped down to being about to parse and render. For tag, filter, and block plugins, `liquid_core` will have everything you need.
* Value:
  * Functionality has moved from `Value` to `ValueView`, `ArrayView`, and `ObjectView`.
  * `Date` was renamed to `DateTime`.
  * `DateTime` is now a newtype.
* Library:
  * `liquid` no longer exposes filters, tags, or blocks.  Depend on `liquid_lib` and enable the relevant features to get them.
  * `ParserBuilder`s `extra_filters` and `jekyll_filters` are no more.  Instead depend on `liquid_lib`, enable the `extras`, `shopify`, or `jekyll` features and manually add them.
  * `ParserBuilder`s `with_liquid` and `liquid` have been renamed to `with_stdlib` and `stdlib`.
  * `include` tag was a hybrid of jekyll and liquid styles.  Now there are separate jekyll and luquid plugins.

#### Features

* Value
  * Scalar extended with a date-only type (#253 fixed in #363).
  * Support structs being `ObjectView` (#379).
  * `to_scalar` and `to_object` functions along with existing `to_value` (#381).
  * `scalar`, `array`, and `object` macros along with existing `value` (#381).
  * derive macros for `ObjectView` / `ValueView` (#385).
  * Support `.size` (#136 fixed in #390).
* Parser:
  * Initial reflection support (#357).
* Render:
  * Make accessing variables faster (#386).
* Filters:
  * Support the `where` filter (#291 fixed in #349)
  * `sort`, `sort_natural`, `compact` now accept `property` parameter (#333, #334, #335 fixed in #352).

#### Fixes

* Reflection
  * `'static` lifetimes were relaxed (#367).
* Filters:
  * `sort` order of `nil` was incorrect (#262 fixed in #352).
  * `sort` should work on scalars (#250 fixed in #352).

## 0.19.0 - 2019-06-08


#### Features

*   Reflection for tags/blocks ([72f8cee8](https://github.com/cobalt-org/liquid-rust/commit/72f8cee870b34ba76b0297e8e8e012f9ba88427c), closes [#315](https://github.com/cobalt-org/liquid-rust/issues/315), breaks [#](https://github.com/cobalt-org/liquid-rust/issues/))
* **Revamp Filters API:**
  *  New Filter API ([7a7de4b5](https://github.com/cobalt-org/liquid-rust/commit/7a7de4b540c6625da0bc3e432a20884daae0bdf1), closes [#301](https://github.com/cobalt-org/liquid-rust/issues/301))
  *  Create liquid-derive crate ([c05525dd](https://github.com/cobalt-org/liquid-rust/commit/c05525dd977b1e04d770fabc33636c436c11673b))
  *  Add named arguments in grammar ([abdd5cb4](https://github.com/cobalt-org/liquid-rust/commit/abdd5cb4b239570f10c6bf0341b207694281319c), closes [#92](https://github.com/cobalt-org/liquid-rust/issues/92))

#### Breaking Changes

*   Reflection for tags/blocks ([72f8cee8](https://github.com/cobalt-org/liquid-rust/commit/72f8cee870b34ba76b0297e8e8e012f9ba88427c), closes [#315](https://github.com/cobalt-org/liquid-rust/issues/315), breaks [#](https://github.com/cobalt-org/liquid-rust/issues/))
*  New Filter API ([7a7de4b5](https://github.com/cobalt-org/liquid-rust/commit/7a7de4b540c6625da0bc3e432a20884daae0bdf1), closes [#301](https://github.com/cobalt-org/liquid-rust/issues/301))



## 0.18.2 - 2019-02-01


#### Features

* **jekyll-filter:**  slugify filter ([21a5be0b](https://github.com/cobalt-org/liquid-rust/commit/21a5be0b0f538ae67d31e5a23180f88af95df69d))



## 0.18.1 - 2019-01-23


#### Bug Fixes

* **comment:**  parse tags inside comment, but ignore their content ([a153b127](https://github.com/cobalt-org/liquid-rust/commit/a153b12775bc0d8c23f23905da60ea2c8f21dbee))
* **grammar:**  allow unmatched `}}` and `%}` as valid liquid ([1889c7b0](https://github.com/cobalt-org/liquid-rust/commit/1889c7b09e19f315e470ff2e70a06e503759eaa0), closes [#320](https://github.com/cobalt-org/liquid-rust/issues/320))
* **parser:**  blocks can accept invalid liquid ([3b2b5fcc](https://github.com/cobalt-org/liquid-rust/commit/3b2b5fcccd0bdec041ce09da9619cd837a81af88), closes [#277](https://github.com/cobalt-org/liquid-rust/issues/277))



## 0.18.0 - 2018-12-27


#### Behavior Features

*   Indexing by variable ([c216a439](https://github.com/cobalt-org/liquid-rust/commit/c216a439d978bedb88ec4baba0e8703d6877e20e), closes [#209](https://github.com/cobalt-org/liquid-rust/issues/209))
* **array:**  indexing with `.first` and `.last` ([36d79cf2](https://github.com/cobalt-org/liquid-rust/commit/36d79cf2f5855b2f5428d8a2d173af8d26dd98bf))
* **case_block:**  support comma separated values in `when` ([0e56f772](https://github.com/cobalt-org/liquid-rust/commit/0e56f7721863a8065ee33eb035b343e16d04e231))
* **errors:**  Report available tags blocks ([04e486a8](https://github.com/cobalt-org/liquid-rust/commit/04e486a87781c6cb46c8ca0da56f7a310eb567cb), closes [#183](https://github.com/cobalt-org/liquid-rust/issues/183))
* **filters:**
  *  array manipulation filters ([94e66600](https://github.com/cobalt-org/liquid-rust/commit/94e6660040d5b021e0ee6aac86ef51e25dd2c725))
  *  Allow the input to index ([ceccb9b2](https://github.com/cobalt-org/liquid-rust/commit/ceccb9b28f68ea168a75daa13059e776ca0d880c), closes [#207](https://github.com/cobalt-org/liquid-rust/issues/207))
  *  add filters "at_least" and "at_most" ([be3e55c0](https://github.com/cobalt-org/liquid-rust/commit/be3e55c079fe43f8f35ebe5add00fd05ef912f79))
* **for-block:**
  * Support iterating on Object ([2469bfc0](https://github.com/cobalt-org/liquid-rust/commit/2469bfc0678b75e16e466cc73b7761b1eb27658d), closes [#201](https://github.com/cobalt-org/liquid-rust/issues/201))
  *  support parameters with variables ([7376ccf5](https://github.com/cobalt-org/liquid-rust/commit/7376ccf51f5d2f0a67ca0dbad3d25d395f4fbd6d), closes [#162](https://github.com/cobalt-org/liquid-rust/issues/162))
* **if_block:**  support multiple conditions with `and` and `or` ([fb16a066](https://github.com/cobalt-org/liquid-rust/commit/fb16a066eeb7fd226883fdae64ec34660d8e539d))
* **tablerow:**  add tablerow object for its tag ([6b95cca5](https://github.com/cobalt-org/liquid-rust/commit/6b95cca5486e2b351bda5b8ffd9e23df98a478d1))
* **unless_block:**  support `else` ([8577eb1e](https://github.com/cobalt-org/liquid-rust/commit/8577eb1e63091623e80da732e3bbf4f6106329d1))
* **tags:**  add more shopify tags ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))
* **grammar:**
  *  Support for empty/blank literals ([ef721815](https://github.com/cobalt-org/liquid-rust/commit/ef72181575e46c7c9329deb30ecb651cab903d3f), closes [#222](https://github.com/cobalt-org/liquid-rust/issues/222)
  *  Support for nil literals ([7d3b0e5b](https://github.com/cobalt-org/liquid-rust/commit/7d3b0e5bb82bef9cc82698fc9514a47a557fdb6f), closes [#223](https://github.com/cobalt-org/liquid-rust/issues/223)
*   Improve error reporting ([e373b1e1](https://github.com/cobalt-org/liquid-rust/commit/e373b1e1a7a1597fe29b5c9020beae1612fc1002))

#### Bug Fixes

*   Deeply nested array indexes ([51c3a853](https://github.com/cobalt-org/liquid-rust/commit/51c3a853a74d0b933983e95a2c6f38d1fdf6512d), closes [#230](https://github.com/cobalt-org/liquid-rust/issues/230))
*   Support more expressive indexing ([e579dd3d](https://github.com/cobalt-org/liquid-rust/commit/e579dd3df4465a4d3b42e3d06293f370ef5b750c)
* **for_block:**  make ranges inclusive. ([42055c35](https://github.com/cobalt-org/liquid-rust/commit/42055c356125b0e66524a5ee7a2fa34e4af4397c))
* **if:**  Improve accuracy of contains op ([07452cd3](https://github.com/cobalt-org/liquid-rust/commit/07452cd38f8d8907bd1f5f602439e2ee99021e69))
* **newlines_to_br:**  should preserve newlines ([01904edb](https://github.com/cobalt-org/liquid-rust/commit/01904edbbb2bc8d7279fe82274df939f7e75d857))
* **comment_block:**  allow nesting ([8d1f64e7](https://github.com/cobalt-org/liquid-rust/commit/8d1f64e7a4e749cf402fe77d2321cba4dacc8a6b))
* **errors:**
  * List alternative filters ([406187bc](https://github.com/cobalt-org/liquid-rust/commit/406187bcd1963636d7119d751938a37b6ce5bab5)
  * look of values/variables ([111160c3](https://github.com/cobalt-org/liquid-rust/commit/111160c3898195753cbccbdca693870acdc278dc), closes [#258](https://github.com/cobalt-org/liquid-rust/issues/258))
* **dbg:**  Fix --help for debug tool ([45c5d397](https://github.com/cobalt-org/liquid-rust/commit/45c5d39770081fc88b1570c64b03d56e6d53b45d))

#### Breaking Behavior Changes

* **grammar:**
  *  Support for empty/blank literals ([ef721815](https://github.com/cobalt-org/liquid-rust/commit/ef72181575e46c7c9329deb30ecb651cab903d3f), closes [#222](https://github.com/cobalt-org/liquid-rust/issues/222)
  *  Support for nil literals ([7d3b0e5b](https://github.com/cobalt-org/liquid-rust/commit/7d3b0e5bb82bef9cc82698fc9514a47a557fdb6f), closes [#223](https://github.com/cobalt-org/liquid-rust/issues/223)

#### Performance

*   Slight change for if-existence ([92aaadf5](https://github.com/cobalt-org/liquid-rust/commit/92aaadf5a583adaad08745613842d9b98225a14a))
*   if-existence bypass error reporting cost ([c7fde6f4](https://github.com/cobalt-org/liquid-rust/commit/c7fde6f44c96a23a86b8fb0f29d060e4dec389bc))
*   Improve for-loop ([f4500fdf](https://github.com/cobalt-org/liquid-rust/commit/f4500fdfafa8543b9161fbefba3175acfcf0b23c))
*   Slight speed up for for-over-hash ([8e2ce0e6](https://github.com/cobalt-org/liquid-rust/commit/8e2ce0e66bfebc698589f5ca9c87853f4cc80170))
*   Speed up variable accesses ([f7392486](https://github.com/cobalt-org/liquid-rust/commit/f7392486b540646a05080275b4d4d5cfa507e3c5)
*   Reduce allocations ([cbb1d254](https://github.com/cobalt-org/liquid-rust/commit/cbb1d254b0ed15b32674e7688af6c55cee91c125), closes [#188](https://github.com/cobalt-org/liquid-rust/issues/188)
* **render:**
  * Use a Write ([0093a595](https://github.com/cobalt-org/liquid-rust/commit/0093a595b9dc0f335c3f8eed7a0309123a82b708), closes [#187](https://github.com/cobalt-org/liquid-rust/issues/187)
  * Bypass UTF-8 validation overhead ([c759fc33](https://github.com/cobalt-org/liquid-rust/commit/c759fc335710797c0cfd75c42b0c598d191fc0b6))
  * Default buffer size ([58eec66b](https://github.com/cobalt-org/liquid-rust/commit/58eec66b49a2a7156b745753c50080b8afad6b7a))
* **value:**
  *  Support str's ([e3aae68d](https://github.com/cobalt-org/liquid-rust/commit/e3aae68d672570db89aeea3daf58423b8f9e6bda)
  *  Reduce allocations with `Cow` ([7fd1e62d](https://github.com/cobalt-org/liquid-rust/commit/7fd1e62d7622ea5e61ef52f225b897f318d59b2c)
  * Allow slicing Paths ([9601e30a](https://github.com/cobalt-org/liquid-rust/commit/9601e30a0803261e92a84bfe86699108f13c03d7)

#### API Features

* **parser:**  accept newlines as `WHITESPACE` ([7bec9871](https://github.com/cobalt-org/liquid-rust/commit/7bec9871b47e15e1aa07a40d3de3518bd80303fc), closes [#286](https://github.com/cobalt-org/liquid-rust/issues/286), [#280](https://github.com/cobalt-org/liquid-rust/issues/280))
* **interpreter:**
  * Runtime partials ([0ef46a17](https://github.com/cobalt-org/liquid-rust/commit/0ef46a170aa21c6113830fa62c26e8b708b97fff))
  *  Support runtime include for tags ([5a0854fa](https://github.com/cobalt-org/liquid-rust/commit/5a0854fa3706505fd1b32dc1bf6ecb7292e173a4)
  *  Allow named stack frames ([4c378178](https://github.com/cobalt-org/liquid-rust/commit/4c3781782176c3ea334963c50f52f9119b178017))
  *  Create dedicated Path for indexing into a Value ([a936ba52](https://github.com/cobalt-org/liquid-rust/commit/a936ba5290130d1fe53ccb4b59c2d04744406674))
  * New caching policies ([d2ba7a61](https://github.com/cobalt-org/liquid-rust/commit/d2ba7a61c0af0600bfd8841fc6f12951ae53557c))
  *  Support arbitrary state ([033c9b75](https://github.com/cobalt-org/liquid-rust/commit/033c9b75c7476057f0858e78ec5b93a0a7cb7895)
* **error:**
  *  liquid re-export all error stuff ([808f708e](https://github.com/cobalt-org/liquid-rust/commit/808f708e5ac48853a977a3cdfc98ec968116766e))
  *  Cloneable errors ([e18c68e1](https://github.com/cobalt-org/liquid-rust/commit/e18c68e14cc3a894737ef23e0f07a9363eea5f79))
  *  Improve missing variable errors ([d6e1aea5](https://github.com/cobalt-org/liquid-rust/commit/d6e1aea5ff9f0340c87bc45b4c07fcd34991805c))
* **value:**
  *  Convinience eq/cmp impls ([78f7a952](https://github.com/cobalt-org/liquid-rust/commit/78f7a9522fc9693919777b04bc8400e708870701))
  *  Value literal macro ([ea5ac0aa](https://github.com/cobalt-org/liquid-rust/commit/ea5ac0aaaa976da089fac6b025bfea450e4da852))
  *  Allow moving into constituent types ([bc07812c](https://github.com/cobalt-org/liquid-rust/commit/bc07812ccdb06e0383cc4701831d0fc20d5be654))
  *  Create to_value ([61ae6de6](https://github.com/cobalt-org/liquid-rust/commit/61ae6de625c53171327263bafec954f0ee2a4977))
  * Return error from Globals ([fde1397b](https://github.com/cobalt-org/liquid-rust/commit/fde1397b5d81b015e30379432c8c017acb63c3b3))
  * Rich Gobals API ([385a62fd](https://github.com/cobalt-org/liquid-rust/commit/385a62fd6d362c4cb5a58694dcf073a471920c34)

#### Breaking API Changes

*   Speed up variable accesses ([f7392486](https://github.com/cobalt-org/liquid-rust/commit/f7392486b540646a05080275b4d4d5cfa507e3c5)
*   Allow slicing Paths ([9601e30a](https://github.com/cobalt-org/liquid-rust/commit/9601e30a0803261e92a84bfe86699108f13c03d7)
*   Rich Gobals API ([385a62fd](https://github.com/cobalt-org/liquid-rust/commit/385a62fd6d362c4cb5a58694dcf073a471920c34)
*   Support more expressive indexing ([e579dd3d](https://github.com/cobalt-org/liquid-rust/commit/e579dd3df4465a4d3b42e3d06293f370ef5b750c)
*   Force serde to always be on ([7f1e2027](https://github.com/cobalt-org/liquid-rust/commit/7f1e2027c28cfd8e34e9ac7e98efd50867c5052d)
*   Cleanup each crate's API ([2e4ab661](https://github.com/cobalt-org/liquid-rust/commit/2e4ab66116ee97183b4d712006bba459f02b3b88)
*   Isolate context creation ([00cac8cb](https://github.com/cobalt-org/liquid-rust/commit/00cac8cb1804e3e1c0514673a9ecc23a13a5f006)
*   Isolate Stack state ([3f8e9432](https://github.com/cobalt-org/liquid-rust/commit/3f8e9432e481fb67ee82d1120405f00112f60810)
*   Isolate cycle state ([34dc950a](https://github.com/cobalt-org/liquid-rust/commit/34dc950a3eddd54340910029d6b4bf13e27721b0)
*   Isolate interrupt state ([06596643](https://github.com/cobalt-org/liquid-rust/commit/06596643a46ec40eadba000602f4af7aa81fe2a4)
*   Reduce allocations ([cbb1d254](https://github.com/cobalt-org/liquid-rust/commit/cbb1d254b0ed15b32674e7688af6c55cee91c125), closes [#188](https://github.com/cobalt-org/liquid-rust/issues/188)
* **compiler:**
  *  Rename LiquidOptions ([0442f38c](https://github.com/cobalt-org/liquid-rust/commit/0442f38c83ec12cd93f6e906c4c43d38383d08bb)
  *  Move filters to interpreter ([b9c2ff87](https://github.com/cobalt-org/liquid-rust/commit/b9c2ff87caf5c30ac6fe5520a9e343690d020c7a)
* **context:**  Reduce scope of public API ([866eb0cb](https://github.com/cobalt-org/liquid-rust/commit/866eb0cb3758a919919009a421c92cd8a148906d)
* **error:**
  *  Simplify Result::context API ([6e6f3b5e](https://github.com/cobalt-org/liquid-rust/commit/6e6f3b5eec626a3e9fd325754f36e622f760021e)
  *  Simplify by removing cloning ([bbd71146](https://github.com/cobalt-org/liquid-rust/commit/bbd71146b4b8f7a14f6ac131b981d76f3dae5a5e)
  *  Clean up error API ([6a950048](https://github.com/cobalt-org/liquid-rust/commit/6a9500488bf6c3bbcc0cbc8c09f52f694595cb6f)
* **errors:**  List alternative filters ([406187bc](https://github.com/cobalt-org/liquid-rust/commit/406187bcd1963636d7119d751938a37b6ce5bab5)
* **filter:**  Switch to standard error type ([3d18b718](https://github.com/cobalt-org/liquid-rust/commit/3d18b71865b38d9f0a3aa4722d0a7adad13c2016)
* **interpreter:**
  *  Support runtime include for tags ([5a0854fa](https://github.com/cobalt-org/liquid-rust/commit/5a0854fa3706505fd1b32dc1bf6ecb7292e173a4)
  *  Moving Text closer to use ([6e9f5bec](https://github.com/cobalt-org/liquid-rust/commit/6e9f5bec78ec8295352b4d40230234bdc89966ee)
  *  Rename Globals to ValueStore ([fbe4f2c3](https://github.com/cobalt-org/liquid-rust/commit/fbe4f2c3f4953ccb08ee317d2e4be4755d5d0e62)
  *  Clarify names ([6b96f92b](https://github.com/cobalt-org/liquid-rust/commit/6b96f92be169aad4d9393cb48d9ad37e856d014a)
* **perf:**  Don't clone globals ([fbc1c153](https://github.com/cobalt-org/liquid-rust/commit/fbc1c153df9988db09cbb8a0a148c8a30b9ab598)
* **plugins:**
  *  Support arbitrary state ([033c9b75](https://github.com/cobalt-org/liquid-rust/commit/033c9b75c7476057f0858e78ec5b93a0a7cb7895)
  *  Abstract plugin registry ([ef4cabf3](https://github.com/cobalt-org/liquid-rust/commit/ef4cabf3c7cdf885bccd1f61d9dfa3da9aab5fa2)
* **render:**  Use a Write ([0093a595](https://github.com/cobalt-org/liquid-rust/commit/0093a595b9dc0f335c3f8eed7a0309123a82b708), closes [#187](https://github.com/cobalt-org/liquid-rust/issues/187)
* **value:**
  *  Newtype for Map ([eab6f40f](https://github.com/cobalt-org/liquid-rust/commit/eab6f40fdb1a6f5858b0dbe3cf89d89600559fd1)
  *  Support str's ([e3aae68d](https://github.com/cobalt-org/liquid-rust/commit/e3aae68d672570db89aeea3daf58423b8f9e6bda)
  *  Reduce allocations with `Cow` ([7fd1e62d](https://github.com/cobalt-org/liquid-rust/commit/7fd1e62d7622ea5e61ef52f225b897f318d59b2c)



## 0.17.1 - 2018-11-17


#### Bug Fixes

*   Deeply nested array indexes ([51c3a853](https://github.com/cobalt-org/liquid-rust/commit/51c3a853a74d0b933983e95a2c6f38d1fdf6512d), closes [#230](https://github.com/cobalt-org/liquid-rust/issues/230))
* **error:**  Improve error reporting ([e372470d](https://github.com/cobalt-org/liquid-rust/commit/e372470d2ed030a4294b7781ce8e80b8041ce673))

#### Features

* **array:**
  * indexing with `.first` and `.last` ([36d79cf2](https://github.com/cobalt-org/liquid-rust/commit/36d79cf2f5855b2f5428d8a2d173af8d26dd98bf))
  * array manipulation filters ([94e66600](https://github.com/cobalt-org/liquid-rust/commit/94e6660040d5b021e0ee6aac86ef51e25dd2c725))
* **blocks:**
  * support multiple if conditions with `and` and `or` ([fb16a066](https://github.com/cobalt-org/liquid-rust/commit/fb16a066eeb7fd226883fdae64ec34660d8e539d))
  * add tablerow object for its tag ([6b95cca5](https://github.com/cobalt-org/liquid-rust/commit/6b95cca5486e2b351bda5b8ffd9e23df98a478d1))



## 0.17.0 - 2018-10-18


#### Breaking Changes

*   Support more expressive indexing ([e579dd3d](https://github.com/cobalt-org/liquid-rust/commit/e579dd3df4465a4d3b42e3d06293f370ef5b750c)
* **for_block:**  make ranges inclusive. ([42055c35](https://github.com/cobalt-org/liquid-rust/commit/42055c356125b0e66524a5ee7a2fa34e4af4397c))

#### Features

* Indexing by variable ([c216a439](https://github.com/cobalt-org/liquid-rust/commit/c216a439d978bedb88ec4baba0e8703d6877e20e), closes [#209](https://github.com/cobalt-org/liquid-rust/issues/209))
* **for_block:**  support parameters with variables ([7376ccf5](https://github.com/cobalt-org/liquid-rust/commit/7376ccf51f5d2f0a67ca0dbad3d25d395f4fbd6d), closes [#162](https://github.com/cobalt-org/liquid-rust/issues/162))
* **filters:**
  * Added: "at_most" ([be3e55c0](https://github.com/cobalt-org/liquid-rust/commit/be3e55c079fe43f8f35ebe5add00fd05ef912f79))
  * Added: "at_least" ([be3e55c0](https://github.com/cobalt-org/liquid-rust/commit/be3e55c079fe43f8f35ebe5add00fd05ef912f79))
* **tags:**
  * Added tablerow ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))
  * Added ifchanged ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))
  * Added increment ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))
  * Added decrement ([18660736](https://github.com/cobalt-org/liquid-rust/commit/1866073682eb97a62bae8eefef87dcc44740a0e6), closes [#163](https://github.com/cobalt-org/liquid-rust/issues/163))

#### Bug Fixes

* **for_block:**  make ranges inclusive. ([42055c35](https://github.com/cobalt-org/liquid-rust/commit/42055c356125b0e66524a5ee7a2fa34e4af4397c))



## 0.16.1 - 2018-10-05




## 0.16.0 - 2018-10-04


#### Breaking Changes

* Force serde to always be on ([7f1e2027](https://github.com/cobalt-org/liquid-rust/commit/7f1e2027c28cfd8e34e9ac7e98efd50867c5052d))
* **context:**  Reduce scope of public API ([866eb0cb](https://github.com/cobalt-org/liquid-rust/commit/866eb0cb3758a919919009a421c92cd8a148906d))
  * Isolate context creation ([00cac8cb](https://github.com/cobalt-org/liquid-rust/commit/00cac8cb1804e3e1c0514673a9ecc23a13a5f006))
  * Isolate Stack state ([3f8e9432](https://github.com/cobalt-org/liquid-rust/commit/3f8e9432e481fb67ee82d1120405f00112f60810))
  * Isolate cycle state ([34dc950a](https://github.com/cobalt-org/liquid-rust/commit/34dc950a3eddd54340910029d6b4bf13e27721b0))
  * Isolate interrupt state ([06596643](https://github.com/cobalt-org/liquid-rust/commit/06596643a46ec40eadba000602f4af7aa81fe2a4))
* **error:**  Clean up error API ([6a950048](https://github.com/cobalt-org/liquid-rust/commit/6a9500488bf6c3bbcc0cbc8c09f52f694595cb6f))
  * **filter:**  Switch to standard error type ([3d18b718](https://github.com/cobalt-org/liquid-rust/commit/3d18b71865b38d9f0a3aa4722d0a7adad13c2016))
* Cleanup each crate's API ([2e4ab661](https://github.com/cobalt-org/liquid-rust/commit/2e4ab66116ee97183b4d712006bba459f02b3b88))
  * Reduce allocations ([cbb1d254](https://github.com/cobalt-org/liquid-rust/commit/cbb1d254b0ed15b32674e7688af6c55cee91c125), closes [#188](https://github.com/cobalt-org/liquid-rust/issues/188))
* **interpreter:**  Clarify names ([6b96f92b](https://github.com/cobalt-org/liquid-rust/commit/6b96f92be169aad4d9393cb48d9ad37e856d014a))
* **perf:**  Don't clone globals ([fbc1c153](https://github.com/cobalt-org/liquid-rust/commit/fbc1c153df9988db09cbb8a0a148c8a30b9ab598))
* **render:**  Use a Write ([0093a595](https://github.com/cobalt-org/liquid-rust/commit/0093a595b9dc0f335c3f8eed7a0309123a82b708), closes [#187](https://github.com/cobalt-org/liquid-rust/issues/187))
* **value:**
  *  Support str's ([e3aae68d](https://github.com/cobalt-org/liquid-rust/commit/e3aae68d672570db89aeea3daf58423b8f9e6bda))
  *  Reduce allocations with `Cow` ([7fd1e62d](https://github.com/cobalt-org/liquid-rust/commit/7fd1e62d7622ea5e61ef52f225b897f318d59b2c))

#### Features

* **for-block:**  Support iterating on Object ([2469bfc0](https://github.com/cobalt-org/liquid-rust/commit/2469bfc0678b75e16e466cc73b7761b1eb27658d), closes [#201](https://github.com/cobalt-org/liquid-rust/issues/201))
* **value:**  Create `to_value` ([61ae6de6](https://github.com/cobalt-org/liquid-rust/commit/61ae6de625c53171327263bafec954f0ee2a4977))

#### Performance

* Reduce allocations ([cbb1d254](https://github.com/cobalt-org/liquid-rust/commit/cbb1d254b0ed15b32674e7688af6c55cee91c125), closes [#188](https://github.com/cobalt-org/liquid-rust/issues/188))
* **render:**  Use a Write ([0093a595](https://github.com/cobalt-org/liquid-rust/commit/0093a595b9dc0f335c3f8eed7a0309123a82b708), closes [#187](https://github.com/cobalt-org/liquid-rust/issues/187))
* **value:**  Support `&'static str`'s ([e3aae68d](https://github.com/cobalt-org/liquid-rust/commit/e3aae68d672570db89aeea3daf58423b8f9e6bda)) ([7fd1e62d](https://github.com/cobalt-org/liquid-rust/commit/7fd1e62d7622ea5e61ef52f225b897f318d59b2c))


## 0.15.0 - 2018-07-30


#### Breaking Changes

*   Upgrade from f32 to f64 ([3eddded2](https://github.com/cobalt-org/liquid-rust/commit/3eddded24056c9f5c2d2d2f3adf143809fe82507))

#### Features

*   Expose filters/tags ([027a67cc](https://github.com/cobalt-org/liquid-rust/commit/027a67cccb9b40ffac0e25d5d9cd4501bdbe4d63))
*   Upgrade from f32 to f64 ([3eddded2](https://github.com/cobalt-org/liquid-rust/commit/3eddded24056c9f5c2d2d2f3adf143809fe82507))
* **date:**  Support today/now ([6a1e0a0f](https://github.com/cobalt-org/liquid-rust/commit/6a1e0a0f3ddc7892e8c84597929dbebc4dd80d29), closes [#181](https://github.com/cobalt-org/liquid-rust/issues/181))



## 0.14.3 - 2018-04-10


#### Bug Fixes

*   Reduce deps for users ([41e9b01a](https://github.com/cobalt-org/liquid-rust/commit/41e9b01a6b2925562b2ef073a8a420c64f08e570))
* **error:**  Make API consumable by failure ([54be3400](https://github.com/cobalt-org/liquid-rust/commit/54be3400dcebe4944196a36be9c99a5187a6f550))



## 0.14.2 - 2018-03-16


#### Features

* **if:**  Bare if is an existence check ([7ab091ca](https://github.com/cobalt-org/liquid-rust/commit/7ab091cadce48d4cb066b3c494fd26f34f0d9625))



## 0.14.1 - 2018-01-24


#### Features

* **API:**
  *  Support &String->Scalar ([b87c983c](https://github.com/cobalt-org/liquid-rust/commit/b87c983c1b5fc9061c1d86424b135119d82fe737))
  *  Re-export datetime ([1ca16f5a](https://github.com/cobalt-org/liquid-rust/commit/1ca16f5a90769e427f45e743fdbfd47629e1d178))



## 0.14.0 - 2018-01-22


#### Features

API
* **Value:**  Control key order ([7ff0fcd0](https://github.com/cobalt-org/liquid-rust/commit/7ff0fcd04d2570aea5338e03128de62e494bee62), closes [#159](https://github.com/cobalt-org/liquid-rust/issues/159))

Users
* **errors:**
  *  Provide context on compile errors ([c17df1f3](https://github.com/cobalt-org/liquid-rust/commit/c17df1f30b2eec8c0ded04919c73c9d5a1d63377))
  *  Report context during render ([73e26cf7](https://github.com/cobalt-org/liquid-rust/commit/73e26cf786a5883a5fe98d66678134392f107cda), closes [#105](https://github.com/cobalt-org/liquid-rust/issues/105))
* **filters:**  Implement basic `compact` support ([c0eadd5c](https://github.com/cobalt-org/liquid-rust/commit/c0eadd5c384e6d5745036ed344116d137043c154))

#### Breaking Changes

API
*   Reduce string cloning ([3d93928b](https://github.com/cobalt-org/liquid-rust/commit/3d93928b2a9ac378dbb3ca8bd097b1ed7112620f)

Users
* **value:**  Improve value coercion practices ([ebb4f40e](https://github.com/cobalt-org/liquid-rust/commit/ebb4f40e315c280825e74cad60b4cd91bbe06ea0), closes [#99](https://github.com/cobalt-org/liquid-rust/issues/99)

#### Performance

*   Reduce string cloning ([3d93928b](https://github.com/cobalt-org/liquid-rust/commit/3d93928b2a9ac378dbb3ca8bd097b1ed7112620f)

#### Bug Fixes

API
*   Remove warning when no-default-features ([8c43de87](https://github.com/cobalt-org/liquid-rust/commit/8c43de871b437d14ac8da14d283bc906c6dea9f2))

Users
* **filters:**  date_in_tz can't parse cobalt date ([1dae5276](https://github.com/cobalt-org/liquid-rust/commit/1dae52767680c7a2b628f631078a97d1ef37ca37))
* **value:**  Improve value coercion practices ([ebb4f40e](https://github.com/cobalt-org/liquid-rust/commit/ebb4f40e315c280825e74cad60b4cd91bbe06ea0), closes [#99](https://github.com/cobalt-org/liquid-rust/issues/99)



## 0.13.7 - 2018-01-10


#### Features

*   Implement `contains` operator ([a0d27205](https://github.com/cobalt-org/liquid-rust/commit/a0d2720570d13d489d7d929452c41334a9d019eb), closes [#155](https://github.com/cobalt-org/liquid-rust/issues/155))



## 0.13.6 - 2017-12-29


#### Features

* **filters:**  date can parse YYYY-MM-DD HH:MM:SS TTTT ([59ab76dc](https://github.com/cobalt-org/liquid-rust/commit/59ab76dcd343a6d9d0fff497e6ba2ff1140b3f2a))



## 0.13.5 - 2017-12-27

* Update dependencies

## 0.13.4 - 2017-12-27


#### Bug Fixes

* **parse:**  Error on empty expressions ([5cffe44a](https://github.com/cobalt-org/liquid-rust/commit/5cffe44a5fb3821dab8a41b8662596421f387659), closes [#139](https://github.com/cobalt-org/liquid-rust/issues/139))
* **raw:**  Stop swapping the text's order ([bd45c14b](https://github.com/cobalt-org/liquid-rust/commit/bd45c14b58e1b22e156b42f3c5629e3a0692e7d4), closes [#79](https://github.com/cobalt-org/liquid-rust/issues/79))



## 0.13.3 - 2017-12-18


#### Bug Fixes

* **for:**  Re-enable support for object.access ([cc9998b5](https://github.com/cobalt-org/liquid-rust/commit/cc9998b55a225941fc5d402f414c32abf64c4500))



## 0.13.2 - 2017-12-18


#### Features

* **api:**  Add missing traits ([e0f82705](https://github.com/cobalt-org/liquid-rust/commit/e0f82705e25e7ff40d246749e7d8b0da04637813))

#### Bug Fixes

* **nil:**  Equality logic missed a case ([111d10a6](https://github.com/cobalt-org/liquid-rust/commit/111d10a695eaf8d906c77569aac627042d52f8eb))



## 0.13.1 - 2017-12-17

Minor docs change.


## 0.13.0 - 2017-12-17


#### Features

* **api:**  Make Renderable debuggable ([802b0af0](https://github.com/cobalt-org/liquid-rust/commit/802b0af0045874565d68a4c4f3b957ddef1b44bd))

#### Bug Fixes

* **dbg:**  Remove debug code ([7bf2a3d4](https://github.com/cobalt-org/liquid-rust/commit/7bf2a3d4754252a0c67c7c514e1dca542e565e4c))
* **for:**  Remove non-standard for_loop variable ([0d9515fe](https://github.com/cobalt-org/liquid-rust/commit/0d9515fe1a8c89e9604beb1a69370256d0f23f08))

#### Breaking Changes

* **for:**  Remove non-standard for_loop variable ([0d9515fe](https://github.com/cobalt-org/liquid-rust/commit/0d9515fe1a8c89e9604beb1a69370256d0f23f08))



## 0.12.0 - 2017-11-29


#### Features

*   Make LiquidOptions cloneable ([838e5261](https://github.com/cobalt-org/liquid-rust/commit/838e5261b6654aab2a93cb5ff2220f75e2d554df))
  *   Make TemplateRepository cloneable ([94f337ae](https://github.com/cobalt-org/liquid-rust/commit/94f337aee53cdd126001b32427b415b20d70d25a))
  *   Make ParseBlock cloneable ([472fb638](https://github.com/cobalt-org/liquid-rust/commit/472fb638e79ab1126979aecb258990d4b93f2935))
  *   Make ParseTag cloneable ([ec59839d](https://github.com/cobalt-org/liquid-rust/commit/ec59839d9d1deff52bb663d0310d5efbca5acace))

#### Breaking Change

*   Make TemplateRepository cloneable ([94f337ae](https://github.com/cobalt-org/liquid-rust/commit/94f337aee53cdd126001b32427b415b20d70d25a))
*   Make ParseBlock cloneable ([472fb638](https://github.com/cobalt-org/liquid-rust/commit/472fb638e79ab1126979aecb258990d4b93f2935))
*   Make ParseTag cloneable ([ec59839d](https://github.com/cobalt-org/liquid-rust/commit/ec59839d9d1deff52bb663d0310d5efbca5acace))


## 0.11.0 - 2017-11-08


#### Features

* **syntax:** Add `arr[0]` and `obj["name"]` indexing (PR #141, fixes #127)
* **value:**  Add nil value to support foreign data (PR #140, [89f6660d](https://github.com/cobalt-org/liquid-rust/commit/89f6660d61ee3a59d3e29e7ad8fe6b31791b1d6f))

#### Breaking Change

* **value:**  Add nil value to support foreign data (PR #140, [89f6660d](https://github.com/cobalt-org/liquid-rust/commit/89f6660d61ee3a59d3e29e7ad8fe6b31791b1d6f))
  * Technically will break anyone matching on `liquid::Value`.

## 0.10.1 - 2017-09-24


#### Features

*   Turn `serde` into a default feature. ([6be99f1d](https://github.com/cobalt-org/liquid-rust/commit/6be99f1da4c066dc08eafd6918f604409f93d43d), closes [#128](https://github.com/cobalt-org/liquid-rust/issues/128))

### Bug Fixes
* Stop recompiling everytime due to Skeptic.


## v0.10.0 - 2017-05-27


#### Features

* **filters:**
  *  Add sort_natural ([ef14f871](https://github.com/cobalt-org/liquid-rust/commit/ef14f87151d73e6079450ec46ebd9da805966aa7))
  *  Implement a dummy `compact` ([44d4d061](https://github.com/cobalt-org/liquid-rust/commit/44d4d0619754fbce519a8d51743651d4cee8e00d))
  *  map filter ([52dc03c0](https://github.com/cobalt-org/liquid-rust/commit/52dc03c06a25a037cc65da3f39f46711be62d76c))
  *  Add concat filter ([36d0d2c1](https://github.com/cobalt-org/liquid-rust/commit/36d0d2c1c4250fa16a3a16af2754ba14f6adb62d))
  *  `round` accepts a precision param ([ef691f13](https://github.com/cobalt-org/liquid-rust/commit/ef691f137d6327df7479abd68ae165f282da2aff))
* **Value:**
  *  Add serde support ([8ae7f5a1](https://github.com/cobalt-org/liquid-rust/commit/8ae7f5a1da00434a6c4d7297938164452d943f09) and [839f44b3](https://github.com/cobalt-org/liquid-rust/commit/839f44b3bdce926c8520d77e9a9e35b60d8e522a), closes [#113](https://github.com/cobalt-org/liquid-rust/issues/113))
  *  Add convenience functions ([4b73b3c2](https://github.com/cobalt-org/liquid-rust/commit/4b73b3c2ebb2a48c05052adff8a104187d58943f))
  *  Publicly expose Object and Array ([280c6d99](https://github.com/cobalt-org/liquid-rust/commit/280c6d9956347f7903e719cb55ee14da46ce1465))
* **debug:**  Adding CLI for testing liquid ([171cbfe0](https://github.com/cobalt-org/liquid-rust/commit/171cbfe0ba297c496dbb738ba136b8d6cbce9eb7) and [9d4b4088](https://github.com/cobalt-org/liquid-rust/commit/9d4b408881292cb57c858d144b91a3f626e53f05))
* **performance:**  Add benchmarks ([0e90972d](https://github.com/cobalt-org/liquid-rust/commit/0e90972d620c02f6e587076e093c330287de070b))

#### Bug Fixes

* **filters:**
  *  Align behavior with shopify/liquid ([ebd7ebc6](https://github.com/cobalt-org/liquid-rust/commit/ebd7ebc696b6176e6a8f24b3efb58f5683d1c341))
  *  Moved `pluralize` to `extra-filters` ([17d57c09](https://github.com/cobalt-org/liquid-rust/commit/17d57c093fc8771531c13b6f587b44b2b25d2b03))

<!-- next-url -->
[Unreleased]: https://github.com/assert-rs/predicates-rs/compare/v0.23.0...HEAD
[0.23.0]: https://github.com/assert-rs/predicates-rs/compare/v0.22.0...v0.23.0
[0.22.0]: https://github.com/assert-rs/predicates-rs/compare/v0.21.5...v0.22.0
[0.21.5]: https://github.com/assert-rs/predicates-rs/compare/v0.21.4...v0.21.5
