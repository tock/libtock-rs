Third Party Dependencies
========================

This document is about dependencies that are required to build applications
using `libtock-rs`. These dependencies are not contained in the libtock-rs
repository, but are used by libtock-rs when libtock-rs is used as a dependency
of an application. Dependencies required to run `libtock-rs`' tests (such as
`make`) are outside the scope of this document.

## Unaudited Required Dependencies

`libtock-rs` has the following required build dependencies, none of which are
currently audited:

* The Rust toolchain, including
  [`cargo`](https://github.com/rust-lang/cargo),
  [`rustc`](https://github.com/rust-lang/rust/tree/master/src/rustc), and
  [`libcore`](https://github.com/rust-lang/rust/tree/master/src/libcore). The
  specific toolchain version used is specified by the `rust-toolchain` file at
  the root of the repository.
* [`syn`](https://crates.io/crates/syn), pulled in by `libtock_codegen`.
* [`quote`](https://crates.io/crates/quote), pulled in by `libtock_codegen`.
* [`proc-macro2`](https://crates.io/crates/proc-macro2), pulled in by
  `libtock_codegen`.

## Avoiding Optional Dependencies

To avoid pulling in optional dependencies, users should use `libtock_core`
instead of `libtock`. `libtock_core` is in the `core/` directory.
