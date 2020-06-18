# safe-bytes

This crate allows reading bytes representation of structs
even in presence of padding bytes.

[![crates](https://img.shields.io/crates/v/safe-bytes.svg?label=safe-bytes)](https://crates.io/crates/safe-bytes)
[![docs](https://docs.rs/safe-bytes/badge.svg)](https://docs.rs/safe-bytes)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](LICENSE-APACHE)

Simply derive [`SafeBytes`] for structures
where all field types are [`SafeBytes`] implementations.
And [`SafeBytes::safe_bytes`] would initialize all padding bytes
before returning `&[u8]`.
All primitives implement [`SafeBytes`] as there is no padding bytes.
Additionally some std types implement [`SafeBytes`].

Note that in order to initialize padding bytes
[`SafeBytes::safe_bytes`] takes mutable reference `&mut self`.
And returns shareable reference `&[u8]` because not all
bitpatterns may be allowed for the type.

[`SafeBytes`]: https://docs.rs/safe-bytes/0.1.0/safe_bytes/trait.SafeBytes.html
[`SafeBytes::safe_bytes`]: https://docs.rs/safe-bytes/0.1.0/safe_bytes/trait.SafeBytes.html#tymethod.safe_bytes

## License

This repository is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution Licensing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
