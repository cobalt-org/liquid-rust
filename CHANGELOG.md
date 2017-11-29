<a name="0.12.0"></a>
## 0.12.0 (2017-11-29)


#### Features

*   Make LiquidOptions cloneable ([838e5261](https://github.com/cobalt-org/liquid-rust/commit/838e5261b6654aab2a93cb5ff2220f75e2d554df))
  *   Make TemplateRepository cloneable ([94f337ae](https://github.com/cobalt-org/liquid-rust/commit/94f337aee53cdd126001b32427b415b20d70d25a))
  *   Make ParseBlock cloneable ([472fb638](https://github.com/cobalt-org/liquid-rust/commit/472fb638e79ab1126979aecb258990d4b93f2935))
  *   Make ParseTag cloneable ([ec59839d](https://github.com/cobalt-org/liquid-rust/commit/ec59839d9d1deff52bb663d0310d5efbca5acace))

#### Breaking Change

*   Make TemplateRepository cloneable ([94f337ae](https://github.com/cobalt-org/liquid-rust/commit/94f337aee53cdd126001b32427b415b20d70d25a))
*   Make ParseBlock cloneable ([472fb638](https://github.com/cobalt-org/liquid-rust/commit/472fb638e79ab1126979aecb258990d4b93f2935))
*   Make ParseTag cloneable ([ec59839d](https://github.com/cobalt-org/liquid-rust/commit/ec59839d9d1deff52bb663d0310d5efbca5acace))


<a name="0.11.0"></a>
## 0.11.0 (2017-11-08)


#### Features

* **syntax:** Add `arr[0]` and `obj["name"]` indexing (PR #141, fixes #127)
* **value:**  Add nil value to support foreign data (PR #140, [89f6660d](https://github.com/cobalt-org/liquid-rust/commit/89f6660d61ee3a59d3e29e7ad8fe6b31791b1d6f))

#### Breaking Change

* **value:**  Add nil value to support foreign data (PR #140, [89f6660d](https://github.com/cobalt-org/liquid-rust/commit/89f6660d61ee3a59d3e29e7ad8fe6b31791b1d6f))
  * Technically will break anyone matching on `liquid::Value`.

<a name="0.10.1"></a>
## 0.10.1 (2017-09-24)


#### Features

*   Turn `serde` into a default feature. ([6be99f1d](https://github.com/cobalt-org/liquid-rust/commit/6be99f1da4c066dc08eafd6918f604409f93d43d), closes [#128](https://github.com/cobalt-org/liquid-rust/issues/128))

### Bug Fixes
* Stop recompiling everytime due to Skeptic.


<a name="v0.10.0"></a>
## v0.10.0 (2017-05-27)


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



