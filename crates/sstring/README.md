SString
===========

> Immutable string type optimized for static data.

[![Build Status](https://dev.azure.com/cobalt-org/cobalt-org/_apis/build/status/liquid-rust?branchName=master)](https://dev.azure.com/cobalt-org/cobalt-org/_build/latest?definitionId=1&branchName=master)
[![Crates Status](https://img.shields.io/crates/v/sstring.svg)](https://crates.io/crates/sstring)

Motivation: Allow zero-cost abstraction over structs and maps. Struct field
names are effectively `'static`.  To avoid allocating when referring to them,
this crate preserves `&'static str` across strings (`SString`),
references (`SStringRef`), and lifetime abstractions (`SStringCow`).

Additional optimizations:
- Small string optimizations (hash map keys tend to be small)

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
