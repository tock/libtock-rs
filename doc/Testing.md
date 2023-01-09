Testing
=======

This document gives an introduction to the tests in this repository, and is
intended to be useful to `libtock-rs` contributors.

## Unit Tests

Cargo packages in this repository that are platform-independent (i.e. that do
not depend on `libtock_runtime`) have unit tests that use Rust's builtin test
harness. These unit tests run on a fully-featured host OS (Linux, Mac OS,
Windows, etc...) and therefore do not depend on a real Tock kernel. Instead,
these unit tests use the fake kernel provided by `libtock_unittest`. As a
result, most `libtock-rs` crates have `libtock_unittest` in their
`[dev-dependencies]`.

`libtock_platform` is a bit of an exception to this rule. If all of
`libtock_platform`'s unit tests were in `libtock_platform`, the following would
occur:

1. `cargo` builds `libtock_platform` with `cfg(test)` enabled to make the test
   binary.
2. `cargo` builds `libtock_unittest` because it is a `[dev-dependency]` of
   `libtock_platform`
3. `cargo` builds `libtock_platform` without `cfg(test)` because it is a
   dependency of `libtock_unittest`

Both copies of `libtock_platform` end up in the final binary. These contain
incompatible instances of the `Syscalls` trait, resulting in hard-to-understand
errors.

To solve this, some of `libtock_platform`'s unit tests (namely, those that
require `libtock_unittest`) were moved to a `platform_test` crate.

## Integration Tests

`libtock-rs`'s integration tests are Tock process binaries that can run on an
emulated or real Tock system. They live in `libtock`'s `tests/` directory.

TODO: Figure out a test runner strategy for automatically running all the
integration tests, and document it here.
