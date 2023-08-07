Coding Style
============

## Naming Conventions

Many things in `libtock-rs` live outside Rust's namespacing system, and can
therefore collide with other libraries. Examples include:

1. `cargo` package names
2. Environment variables
3. Linker script file names

For example, if `libtock-rs` were to contain a `cargo` package called `adc`,
that package would likely collide with another package if we were to try to
upload it to crates.io.

To prevent these collisions, things in `libtock-rs` that can experience external
name collisions should have a `libtock` prefix (with suitable capitalization).
For example:

1. The runtime crate is called `libtock_runtime`
2. The build platform environment variable is `LIBTOCK_PLATFORM`
3. The linker script is called `libtock_layout.ld`

However, this prefix should be omitted when it is unnecessary, to avoid naming
everything in the repository `libtock_*`. For example:

1. `libtock_` is omitted from directory names. `libtock_runtime` lives in the
   `/runtime/` directory.
2. Cargo packages that are not used externally, such as the `print_sizes` tool
   (which is only used for developing `libtock-rs` itself), do not have the
   `libtock_` prefix.
