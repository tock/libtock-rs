[![Build Status](https://travis-ci.org/tock/libtock-rs.svg?branch=master)](https://travis-ci.org/tock/libtock-rs)

# libtock-rs

Rust userland library for Tock (WIP)

Tested with tock [Release 1.4.1](https://github.com/tock/tock/commit/7e37bf67761d83fd585cace4fb201e2864d300b1).

The library works in principle on most boards, but there is currently the [showstopper
bug #28](https://github.com/tock/libtock-rs/issues/28) that prevents
the generation of relocatable code. This means that all applications
must be installed at the flash address they are compiled with, which
usually means that they must be compiled especially for your board
and that there can only be one application written in rust at a time
and it must be installed as the first application on the board, unless
you want to play games with linker scripts.
There are some `boards/layout_*.ld` files provided that allow to run the
examples on common boards.
Due to MPU region alignment issues they may not work for applications
that use a lot of RAM, in that case you may have to change the SRAM
start address to fit your application.

## Getting Started

This project is nascent and still under heavy development, but first steps:

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
    make nrf52 # Builds all examples for the nrf52 platform
    ```

    ```bash
    make opentitan # Builds all examples for the OpenTitan platform
    ```

    ```bash
    make opentitan FEATURES=alloc # Builds all examples for the OpenTitan platform, with alloc feature enabled
    ```

    ```bash
    make flash-hail EXAMPLE=blink # Flash the example 'blink' program to the hail platform
    ```

    For an unknown platform, you may have to create your own memory layout definition. Place the layout definition file at `boards/layout_<platform>.ld` and do not forget to enhance the `tockloader_flags` dispatching section in `tools/flash.sh`. You are welcome to create a PR, s.t. the number of supported platforms grows.

## Using libtock-rs

The easiest way to start using libtock-rs is adding an example to the examples folder.
The boiler plate code you would write is

```rust
#![no_std]

use libtock::result::TockResult;

#[libtock::main]
async fn main() -> TockResult<()> {
  // Your code
}
```

If you want to use heap based allocation you will have to add

```rust
extern crate alloc;
```

to the preamble and store your example in the `examples-alloc` folder.

To build the examples for your board you can use

```shell
make <platform> [FEATURES=alloc]
```

An example can be flashed to your board after the build process by running:

```shell
make flash-<platform> EXAMPLE=<example>
```

This script does the following steps for you:

- cross-compile your program
- create a TAB (tock application bundle)
- if you have a J-Link compatible board connected: flash this TAB to your board (using tockloader)


## License

libtock-rs is licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Submodules have their own licenses.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

The contribution guidelines can be found here: [contribution guidelines](CONTRIBUTING.md)
