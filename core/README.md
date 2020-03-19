# libtock-core

Core crate of `libtock-rs`. It contains the architecture specific code of `libtock-rs`. In particular:

 * the entry point
 * `panic` and `alloc_error` handlers
 * the syscalls
 * the allocator (optional)

It has three important feature flags

 * `alloc` - allow for heap. Enables a linked list allocator.
 * `custom_panic_handler` - disable the default panic  handler and allow definition of a custom one using `#[panic_handler]`
 * `custom_alloc_error_handler` - disable the default alloc error handler and allow definition of a custom one using `#[alloc_error_handler]`

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

The contribution guidelines are identical to those of `libtock-rs` and can be found here: [contribution guidelines](../CONTRIBUTING.md)
