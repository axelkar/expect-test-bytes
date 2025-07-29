# expect-test-bytes

[![Crates.io](https://img.shields.io/crates/v/expect-test-bytes)](https://lib.rs/crates/expect-test-bytes)
[![Documentation](https://docs.rs/expect-test-bytes/badge.svg)](https://docs.rs/expect-test-bytes)
![Crates.io MSRV](https://img.shields.io/crates/msrv/expect-test-bytes)
![Crates.io License](https://img.shields.io/crates/l/expect-test-bytes)

A copy of [expect-test](https://github.com/rust-analyzer/expect-test), a minimalistic snapshot testing library, for bytes and binary data.

Currently only supports files. See also [expect-test#47](https://github.com/rust-analyzer/expect-test/issues/47).

## Example

```rs
let actual = b"example\n";

expect_test_bytes::expect_file!["test_data/example"].assert_eq(actual);
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
