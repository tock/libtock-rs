Overview
======

This document gives an overview of the crates in this repository, and is
intended to be useful to `libtock-rs` newcomers.

## `libtock`

`libtock` provides the default `libtock-rs` experience. It re-exports all of the
drivers `libtock-rs` provides, and provides usable defaults for panic handling
and memory allocation. It should be easy to build a Tock application that only
has one direct dependency: `libtock`.

In order to be easy to use, `libtock` has a hard dependency on
`libtock_runtime`, and therefore cannot be used in a unit test environment. If
you want to unit test code that uses `libtock-rs`, you should depend on the
individual libraries rather than `libtock`.

## Naming convention note

Although these crates have yet to be uploaded to crates.io, they likely will be
uploaded in the future. Therefore, to avoid name collisions, most crates in this
repository have a name that begins with `libtock`. Crates that are only intended
for internal use (e.g. `syscall_tests`, which tests code internal to
`libtock_platform`) do not have the `libtock_` prefix.

The directory names of `libtock_` crates do not contain the `libtock_` prefix.

## Core abstractions: `libtock_platform`

In order to unit test `libtock-rs` code, we need a way to run `libtock-rs` on
our development machines (and in CI). Ironically, that means most crates in
`libtock-rs` are platform independent. `libtock_platform` provides the tools
that allow code to run in both a unit test environment and in real Tock apps. It
consists primarily of the `Syscalls` trait and supporting machinery.

## Syscall implementations: `libtock_runtime` and `libtock_unittest`

In order to run `libtock-rs` code, you need a `libtock_platform::Syscalls`
implementation. Multiple implementations exist that work in different
environments:

* `libtock_runtime` provides a syscall interface that uses a real Tock kernel.
  This is the crate to use in Tock process binaries.
* `libtock_unittest` provides a fake kernel for use in unit tests.

In addition, `libtock_runtime` provides the linker script and Rust runtime
needed to start a Tock process binary. `libtock_unittest` relies on `std` to
provide a runtime.

## Panic handler crates

Each Rust binary must have exactly one panic handler (note that `std` provides a
panic handler for binaries that depend on it). The following crates provide a
`#[panic_handler]` implementation for Tock process binaries:

* `libtock_panic_debug` provides useful diagnostics in the event of a panic, at
  the expense of code size. This is the panic handler used by `libtock`.

## Driver crates

Driver crates provide interfaces to specific Tock APIs in the `/apis` directory.
