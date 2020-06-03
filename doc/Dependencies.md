Third Party Dependencies
========================

## Unaudited Required Dependencies

`libtock-rs` has the following required dependencies, none of which are
currently audited:

* [`libcore`](https://github.com/rust-lang/rust/tree/master/src/libcore),
  included as part of the Rust toolchain and implicitly added by the Rust
  compiler.
* [`syn`](https://crates.io/crates/syn), pulled in by `libtock_codegen`.
* [`quote`](https://crates.io/crates/quote), pulled in by `libtock_codegen`.
* [`proc-macro2`](https://crates.io/crates/proc-macro2), pulled in by
  `libtock_codegen`.

## Avoiding Optional Dependencies

To avoid pulling in optional dependencies, users should use `libtock-core`
instead of `libtock`. `libtock-core` is in the `core/` directory.
