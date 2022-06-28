Using `no_auto_layout`
======================

By default, `libtock_runtime` will read the `LIBTOCK_PLATFORM` environment
variable and automatically configure a suitable linker script for that board.
However, these linker scripts may not be suitable for every use case of
`libtock-rs`. Therefore, `libtock_runtime` has a `no_auto_layout` feature to
disable the automatic linker script selection. This document describes the steps
you need to take to enable `no_auto_layout` and provide your own linker script.

## Enabling the `no_auto_layout` feature

`no_auto_layout` may be enabled for a process binary in `Cargo.toml` as follows:

```toml
[dependencies]
libtock_runtime = { features = ["no_auto_layout"], ... }
```

## Specifying a linker script

You need to pass the `-T` argument to the linker to tell it where to find the
linker script. You can do this by using `-C link-arg=` in `RUSTFLAGS`. For
example, you may put this in your `.cargo/config`:

```toml
[target.'cfg(any(target_arch = "arm", target_arch = "riscv32"))']
rustflags = [
    "-C", "link-arg=-Tyour_layout.ld",
]
```

By default, the linker will search for `your_layout.ld` relative to its working
directory, which is the root of the `cargo` workspace your package is in.
However, you can specify additional search directories as described below.

## `build.rs` support

Merely adding `-C link-arg=-T` to your `RUSTFLAGS` does not tell `cargo` that
your package depends on the linker script. Therefore, `cargo` will not know that
it needs to rebuild your process binary when the linker script changes. This can
result in hard-to-debug issues that disappear upon `cargo clean`. To avoid this,
you need to tell `cargo` to rebuild your process binary when the linker script
changs. You can do this in `build.rs`:

```rust
fn main() {
    println!("cargo:rerun-if-changed=your_layout.ld");
}
```

You can add directories to the linker search path by using
`println!("cargo:rustc-link-search=your_directory");`. Note that the
`rerun-if-changed` path will still be relative to your `cargo` workspace root;
it does not use the search path like `RUSTFLAGS="-C link_arg=-T<layout>"` does.
