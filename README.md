![Build Status](https://github.com/tock/libtock-rs/workflows/ci/badge.svg)

# libtock-rs

Rust userland library for Tock

Generally this library tracks recent Tock master; the kernels it builds to run
examples under emulation are pinned to a specific commit (`TOCK_COMMIT` in
`Makefile.qemu`).

The library should work on all Tock boards, but currently apps must be compiled
for the flash and RAM address they are executed at. See [Fix
relocation](https://github.com/tock/libtock-rs/issues/28) for more details. You
may either compile a process binary especially for your board and use only a
single application written in rust at a time, or use the `make tab` target that
builds examples for a series of likely useful flash and RAM addresses.

## Getting Started

1.  Ensure you have [rustup](https://www.rustup.rs/) installed.

1.  Clone the repository:

    ```shell
    git clone https://github.com/tock/libtock-rs
    cd libtock-rs
    ```

1.  Install the dependencies:

    ```shell
    make setup
    ```

1.  If you want to run examples under emulation, install a QEMU with 32-bit
    RISC-V support: `qemu-system-misc` or, on newer releases, `qemu-system-riscv`
    on Debian and Ubuntu; `qemu-system-riscv` on Fedora; `qemu` on Homebrew.
    `make setup-qemu` reports whether the one you have will do.

1.  Use `make` to build examples

    ```shell
    make nrf52 EXAMPLE=console # Builds the console example for the nrf52
    ```

## Using libtock-rs

The easiest way to start using libtock-rs is adding an example to the
`examples/` folder. We recommend starting by copying the `console` example, as
it is a simple example that shows you how to perform normal debug prints.

### Building for a specific board

To build your example for your board you can use

```shell
make <platform> EXAMPLE=<example>
```

An example can be flashed to your board after the build process by running:

```shell
make flash-<platform> EXAMPLE=<example>
```

This script does the following steps for you:

- cross-compile your program
- create a TAB (tock application bundle)
- if you have a J-Link compatible board connected: flash this TAB to your board (using tockloader)

### Enabling rust-embedded support

libtock-rs can be built to be compatible with the rust-embedded
[embedded_hal](https://docs.rs/embedded-hal/1.0.0/embedded_hal/index.html) by
including the following when running `make`

```shell
FEATURES=rust_embedded
```

If using libtock-rs or a sub-crate as a cargo dependency the `rust_embedded`
can also be enabled via Cargo.

### Running an example under emulation

Some of the supported boards can be emulated, so an example can be run with no
hardware at all. Run `make` to list them, then run one:

```shell
make qemu-example-qemu_rv32_virt EXAMPLE=console
```

This builds a Tock kernel, which needs Tock's source. By default libtock-rs
fetches a pinned revision into a cache directory and manages it for you. To
build against a Tock tree of your own instead — a branch you are working on,
say — point `TOCK_DIR` at it:

```shell
make qemu-example-qemu_rv32_virt EXAMPLE=console TOCK_DIR=$HOME/src/tock
```

A tree you supply this way is used exactly as it is: libtock-rs never fetches
into it, checks anything out in it, or cleans it.

An emulated board runs until you interrupt it with Ctrl+C.

### Building a generic TAB (Tock Application Bundle) file

To build your example for a variety of boards you can use

```shell
make tab EXAMPLE=<example>
```

To install the tab use tockloader

```shell
tockloader install target/tab/<example.tab>
```

Tockloader will determine which compiled version with the correct flash and RAM
addresses to use.


## License

libtock-rs is licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Submodules, as well as the code in the `ufmt` directory, have their own licenses.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

The contribution guidelines can be found here: [contribution guidelines](CONTRIBUTING.md)
