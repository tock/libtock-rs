Miri Tips
=========

`libtock-rs` runs most of its unit tests under
[Miri](https://github.com/rust-lang/miri) to detect undefined behavior. However,
Miri does not detect all undefined behavior encountered. This document lists
techniques for maximizing the amount of undefined behavior identified by Miri.

## Miri flags

Miri has [configuration
flags](https://github.com/rust-lang/miri#miri--z-flags-and-environment-variables),
which can be set using the `MIRIFLAGS` environment variable. We run the tests
both with and without `-Zmiri-track-raw-pointers`, as both versions of Stacked
Borrows has undefined behavior the other does not (see [this
discussion](https://github.com/rust-lang/miri/pull/1748) for more information).
On the run with `-Zmiri-track-raw-pointers`, we also set
`-Zmiri-symbolic-alignment-check` to make Miri's alignment check more pendantic.

## Detecting invalid values

Miri does not always detect undefined behavior when an invalid value is created
but not used. Here are some cases that Miri currently accepts:

1. Unused reference to an invalid value
   (https://github.com/rust-lang/miri/issues/1638)
1. Unused uninitialized value (https://github.com/rust-lang/miri/issues/1340)
1. Slices pointing to invalid values
   (https://github.com/rust-lang/miri/issues/1240)

Note that copying the value around (including passing it to functions) does
*not* count as a use for the purpose of this check.

For types that implement `core::fmt::Debug` (i.e. most types), you can check a
value is valid by attempting to debug-print it:

```rust
format!("{:?}", value);
```

If `value` is invalid, then Miri will identify undefined behavior in the above
code.
