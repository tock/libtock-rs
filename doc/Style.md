Coding Style
============

## List Ordering

Source code tends to contain many lists whose order is unimportant, such as
dependency lists in `Cargo.toml` and `mod` declarations in `.rs` files. When
there isn't a better reason to prefer a particular order, these lists should be
in alphabetical order.

Benefits:

1. When lists become long, this makes it easier to search for a particular
   entry.
2. Always adding new entries at the bottom of the list results in merge
   conflicts whenever two PRs add entries to the same list. Putting them in
   alphabetical order decreases the probability of merge conflicts.

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
