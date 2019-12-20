KString
===========

> Key String: Optimized for map keys.

[![Build Status](https://dev.azure.com/cobalt-org/cobalt-org/_apis/build/status/liquid-rust?branchName=master)](https://dev.azure.com/cobalt-org/cobalt-org/_build/latest?definitionId=1&branchName=master)
[![Crates Status](https://img.shields.io/crates/v/kstring.svg)](https://crates.io/crates/kstring)

Considerations:
- Large maps
- Most keys live and drop without being used in any other way
- Most keys are relatively small (single to double digit bytes)
- Keys are immutable
- Allow zero-cost abstractions between structs and maps (e.g. no allocating
  when dealing with struct field names)

Ramifications:
- Inline small strings rather than going to the heap.
- Preserve `&'static str` across strings (`KString`),
  references (`KStringRef`), and lifetime abstractions (`KStringCow`) to avoid
  allocating for struct field names.

## License

Licensed under either of

 * Apache License, Version 2.0, (http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
