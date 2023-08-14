![Build Status](https://github.com/tock/libtock-rs/workflows/ci/badge.svg)

# libtock-rs

Rust userland library for Tock

Generally this library was tested with Tock [Release
2.1.1](https://github.com/tock/tock/releases/tag/release-2.1.1).

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
    git clone --recursive https://github.com/tock/libtock-rs
    cd libtock-rs
    ```

1.  Install the dependencies:

    ```shell
    make setup
    ```

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
