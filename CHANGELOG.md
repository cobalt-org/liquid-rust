<a name="v0.10.0"></a>
## v0.10.0 (2017-05-27)


#### Features

* **Value:**  Add serde support ([8ae7f5a1](https://github.com/cobalt-org/liquid-rust/commit/8ae7f5a1da00434a6c4d7297938164452d943f09))
* **cargo:**  Add badges ([3210ce0b](https://github.com/cobalt-org/liquid-rust/commit/3210ce0b0a1afebc48e08f06353b0239ad367670))
* **dbg:**  Support yaml/json files via new Serde ([9d4b4088](https://github.com/cobalt-org/liquid-rust/commit/9d4b408881292cb57c858d144b91a3f626e53f05))
* **debug:**  Adding CLI for testing liquid ([171cbfe0](https://github.com/cobalt-org/liquid-rust/commit/171cbfe0ba297c496dbb738ba136b8d6cbce9eb7))
* **filters:**
  *  Add sort_natural ([ef14f871](https://github.com/cobalt-org/liquid-rust/commit/ef14f87151d73e6079450ec46ebd9da805966aa7))
  *  Implement a dummy `compact` ([44d4d061](https://github.com/cobalt-org/liquid-rust/commit/44d4d0619754fbce519a8d51743651d4cee8e00d))
  *  map filter ([52dc03c0](https://github.com/cobalt-org/liquid-rust/commit/52dc03c06a25a037cc65da3f39f46711be62d76c))
  *  Add concat filter ([36d0d2c1](https://github.com/cobalt-org/liquid-rust/commit/36d0d2c1c4250fa16a3a16af2754ba14f6adb62d))
  *  `round` accepts a precision param ([ef691f13](https://github.com/cobalt-org/liquid-rust/commit/ef691f137d6327df7479abd68ae165f282da2aff))
* **performance:**  Add benchmarks ([0e90972d](https://github.com/cobalt-org/liquid-rust/commit/0e90972d620c02f6e587076e093c330287de070b))
* **serde:**  Make serde optional ([839f44b3](https://github.com/cobalt-org/liquid-rust/commit/839f44b3bdce926c8520d77e9a9e35b60d8e522a), closes [#113](https://github.com/cobalt-org/liquid-rust/issues/113))
* **value:**  Add convinience functions ([4b73b3c2](https://github.com/cobalt-org/liquid-rust/commit/4b73b3c2ebb2a48c05052adff8a104187d58943f))

#### Bug Fixes

* **CI:**
  *  Locking down to a specific released rustfmt version ([cbd6dd53](https://github.com/cobalt-org/liquid-rust/commit/cbd6dd53c9c1d0c1ad1820836dfa53e312dff661))
  *  Report tool versions so we can all align ([587389a4](https://github.com/cobalt-org/liquid-rust/commit/587389a456d6c9cc93d6317047e5eb9edff2bb23))
  *  use rustfmt from cargo ([bb3cbbe7](https://github.com/cobalt-org/liquid-rust/commit/bb3cbbe7494d5d564f480fed47e178d02970f777))
  *  Disable caching ([9bf6514f](https://github.com/cobalt-org/liquid-rust/commit/9bf6514f47ef0cdc534f92fe805ff1a53f81cf59))
* **Value:**  Publicly expose Object and Array ([280c6d99](https://github.com/cobalt-org/liquid-rust/commit/280c6d9956347f7903e719cb55ee14da46ce1465))
* **appveyor:**  Correct the path to curl ([23d2f499](https://github.com/cobalt-org/liquid-rust/commit/23d2f4994fe99ef13df2f991f628ee80e6c84fa0))
* **docs:**  markdown formatting for links ([63883caa](https://github.com/cobalt-org/liquid-rust/commit/63883caab32a4e17dabebe7c45e31763e585be4e))
* **filters:**
  *  Align behavior with shopify/liquid ([ebd7ebc6](https://github.com/cobalt-org/liquid-rust/commit/ebd7ebc696b6176e6a8f24b3efb58f5683d1c341))
  *  Moved `pluralize` to `extra-filters` ([17d57c09](https://github.com/cobalt-org/liquid-rust/commit/17d57c093fc8771531c13b6f587b44b2b25d2b03))
* **liquid-dbg:**
  *  address nightly warning ([1a9f9254](https://github.com/cobalt-org/liquid-rust/commit/1a9f925418d964b4a5550300ada2eb7dd06c38d0))
  *  Stricter compilation ([23379ab7](https://github.com/cobalt-org/liquid-rust/commit/23379ab7f1b20b9acb626b671ec652ed301c2eb3))
  *  Resolve clippy complaints ([912f2654](https://github.com/cobalt-org/liquid-rust/commit/912f26540b7d025ad7163fd732276cd1de9c96c7))
* **tests:**
  *  Workaround CI by disabling caching of `target` ([094b0a17](https://github.com/cobalt-org/liquid-rust/commit/094b0a172e3417fc763bcfb02365d70e051e78dd))
  *  Workaround CI by disabling caching of `target` ([a7bd0377](https://github.com/cobalt-org/liquid-rust/commit/a7bd0377a0d61d07822dc1b9386c9ddeff8527c9))



