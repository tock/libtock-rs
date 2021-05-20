Cargo Features
==============

On the surface, `cargo`'s
[features](https://doc.rust-lang.org/cargo/reference/features.html) mechanism
looks great for `libtock-rs`. Process binary crates can depend on `libtock-rs`,
and use `cargo` features to specify which optional `libtock-rs` functionality
they want.

However, `cargo` assumes that it can compile a crate with features that a binary
does not need. This isn't necessarily true for embedded crates, where process
binary authors care about the app size.

## Process Binaries in a Workspace

When used with workspaces, `cargo` features can result in excess code being
built into a repository.

For example, suppose that `libtock` exposes a `malloc` feature, which adds a
dynamic memory allocator. Then if a `libtock` user creates a `cargo` workspace
with the following process binaries in it:

1. `no_malloc`, which depends on `libtock` and does not depend on the `malloc`
   feature.
2. `malloc`, which depends on `libtock` and the `malloc` feature.

With this setup, `malloc` will always work correctly. Also, if you run `cargo
build -p no_malloc`, then `no_malloc` will correctly build without a memory
allocation.

Seems optimal, right?

It's not: if you build the entire workspace with `cargo build --workspace`,
`libtock` will only be built once, with the `malloc` feature! That will result
in a `no_malloc` process binary that contains a memory allocator, which is not
what the author desired.

## Alternative to `cargo` Features

In many cases, `cargo` features can be replaced by splitting a crate up into
smaller crates, and letting process binaries choose what to depend on. For
example, memory allocation can be in a separate crate that process binaries
depend on if and only if they need memory allocation.

Here are some questions to help guide the decision between using a `cargo`
feature and separate crates:

1. Do you forsee a single user writing multiple process binaries, some of which
   use this feature? If yes, then maybe it should not be a `cargo` feature.
2. Will the compiler optimize the feature away entirely if it is included but
   unused? If yes, then making it a `cargo` feature is probably fine.

In many cases, it will make sense to add a `cargo` feature to `libtock` but use
optional crates to implement the feature internally. This way, users with
multiple process binaries in a workspace can choose whether each process binary
depends on the optimal crate.
