Design Considerations
=====================

This document describes several of the factors that constrain the design of
`libtock_core`.

## Size impact

Tock is designed to run on hardware with limited program storage and limited
RAM. On Google's H1 chip, the Tock kernel and apps are limited to a 256 KiB
flash region. The HiFive1 Rev B board has only 16 KiB of RAM.

Note that a Tock system may have multiple process binaries, each of which uses
`libtock_core`, so `libtock_core` must minimize its size impact to the extent
possible.

A process binary is generally stored in non-volatile memory and contains:

  * Runtime headers (the `.crt0_header` section)
  * Executable code (the `.text` section)
  * Read-only data (the `.rodata` section)
  * Non-zero-initialized read-write data (the `.data` section)

A process' memory section is in RAM and contains:

  * Heap memory (if a dynamic memory allocator is present)
  * Zero-initialized read-write data (the `.bss` section)
  * Non-zero-initialized read-write data (the `.data` section)
  * Stack memory

`libtock_core` developers should consider the space usage of their code, and
where the space usage is (non-volatile memory or RAM). Important note: the
`.data` section consumes space in both non-volatile memory and RAM!

Most apps will not use all functionality in `libtock_core`. The size impact of
using `libtock_core` in an app should be commensurate with the `libtock_core`
functionality that app uses. If done correctly, this will allow `libtock_core`'s
users to build Tock systems with multiple small, single-purpose apps without
excessive code duplication.

## Testability

Programmatic tests are important to verify that new functionality works
correctly and to keep existing functionality working through nontrivial
refactorings. Different types of tests offer different capabilities:

  * **Host-based unit tests** run on a non-Tock host OS (such as Linux). They
    can only be used with portable `libtock_core` code, as the code they test
    must run on both Tock and the host OS. These tests must either test code
    that does not directly use system calls, or must direct the system calls to
    a fake Tock kernel. These tests can be fast (compile and run in seconds),
    and can easily simulate a variety of failure modes (e.g. kernel errors) that
    may be difficult to generate with a real Tock kernel.
  * **Emulation tests** run on an emulated Tock system. Currently, emulation
    tests run in QEMU on an emulated HiFive1 Rev B board. These test
    `libtock_core`'s interaction with a real Tock kernel, albeit with limited
    hardware access.
  * **Hardware tests** run on physical hardware with a real Tock kernel. These
    are end-to-end tests that test not only `libtock_core`'s functionality but
    also the Tock kernel's interaction with the real hardware. However,
    automating these tests is difficult, and currently we do not have a way to
    run hardware tests in CI.

If `libtock_core` is functional on real hardware with a real kernel, it should
be able to support emulation tests and hardware tests. However, host-based unit
tests require more work. In its intended use case, `libtock_core` is not
supposed to be portable, supporting only the Tock kernel. Hoever, host-based
unit tests are very valuable, both for rapid development and for testing error
handling code. To support host-based tests, `libtock_core` must be built out of
portable pieces that can be unit tested, even though the library as a whole is
not portable.

Testing pieces of `libtock_core` against a fake kernel can be done using
dependency injection, but most dependency injection techniques have considerable
code size and RAM usage costs. `libtock_core` needs either a dependency
injection technique that has minimal size impact or an alternative mechanism for
testing `libtock_core`'s system call error handling logic.

## Support multiple asynchronous programming models.

The general Rust ecosystem has converged on futures as its building block for
interoperable asynchronous APIs, but futures have a [size
cost](https://github.com/tock/design-explorations/tree/master/size_comparison)
that makes them impractical for some use cases of `libtock_core`. Although
`libtock_core` is still a work in progress (see [issue
217](https://github.com/tock/libtock-rs/issues/217)), it will probably use an
asynchronous API design based on the [Asynchronous Components using Zero Sized
Type
Pointers](https://github.com/tock/design-explorations/tree/master/zst_pointer_async)
exploration.

In order to interoperate with other libraries as well as the `async`/`await`
functionality built in to Rust, we need `libtock_core` to interface well with
futures. In addition, there are several use cases of `libtock_core` that consist
primarily of synchronous code, so we need to be able to use `libtock_core`'s
APIs in a synchronous manner.

The `libtock-c` developers are already familiar with some of the complications
that come up when synchronous code is interfaced to asynchronous code. For
example, calling `printf` -- an extremely common debugging tool -- causes
callbacks to run in the background. This causes surprising and hard-to-debug
reentrancy issues.

## Dependency tree minimization

Some of users of `libtock_core` intend to use it in applications that will
undergo code audits for security certifications. To make this auditing
practical, it is important to allow users to use `libtock_core` without pulling
in a large dependency tree.

As a general rule, a `libtock_core` should avoid having required dependencies
unless those dependencies are minimal (that is, avoiding the dependency would
require reimplementing nearly all of it). Dependencies included in the Rust
toolchain (such as the `core` crate and its dependencies) are an exception, as
they are a part of the language itself.

## No hard `alloc` dependency

It should be possible to use `libtock_core` without bringing in the `alloc`. Of
course, it is fine for `libtock_core` to offer a memory allocator as an optional
feature.
