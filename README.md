[![Build Status](https://travis-ci.org/tock/libtock-rs.svg?branch=master)](https://travis-ci.org/tock/libtock-rs)
# libtock-rs
Rust userland library for Tock (WIP)

Tested with tock [Release 1.4](https://github.com/tock/tock/commit/2cef02405e699c06cefd0aae6a89f0e8cc4395ab).

The library works in principle on most boards, but there is currently the [showstopper
bug #28](https://github.com/tock/libtock-rs/issues/28) that prevents
the generation of relocatable code. This means that all applications
must be installed at the flash address they are compiled with, which
usually means that they must be compiled especially for your board
and that there can only be one application written in rust at a time
and it must be installed as the first application on the board, unless
you want to play games with linker scripts.
There are some `*_layout.ld` files provided that allow to run the
examples on common boards.
Due to MPU region alignment issues they may not work for applications
that use a lot of RAM, in that case you may have to change the SRAM
start address to fit your application.

## Getting Started

This project is nascent and still under heavy development, but first steps:

1. Ensure you have [rustup](https://www.rustup.rs/) installed.

1. Clone the repository.

    ```bash
    git clone https://github.com/tock/libtock-rs
    cd libtock-rs
    ```

1. Install `elf2tab`.

    ```bash
    cargo install -f elf2tab --version 0.4.0
    ```

1. Add dependencies for cross-compilation.

    ```bash
    rustup target add thumbv7em-none-eabi
    ```

1. Use the `run_example` script to compile and run the example app you want
to use:

    ```bash
    ./run_example.sh blink
    ```

    Due to bug #28 this will currently only work if you are using the nRF52-DK platform.

    If you have a nRF52840-DK you must change `link-arg=-Tnrf52_layout.ld` in
    `.cargo/config` to `link-arg=-Tnrf52840_layout.ld`

    If you have a hail board you can flash your device as follows:
     - set the environment variable `hail` to `1`
     - change `link-arg=-Tnrf52_layout.ld` in `.cargo/config` to `link-arg=-Thail_layout.ld`
     - run `run_example.sh` as above.

    For other platforms, you may have to create your own memory layout definition.

## Using libtock-rs

The easiest way to start using libtock-rs is adding an example to the examples folder.
The boiler plate code you would write is
```rust
#![no_std]

extern crate tock;

fn main() {
  // Your code
}
```
If you want to use heap based allocation you will have to add
```rust
#![feature(alloc)]
extern crate alloc;
```
to the preamble.

To run on the code on your board you can use
```bash
./run_example.sh <your app>
```
This script does the following steps for you:
 - cross-compile your program
 - create a TAB (tock application bundle)
 - if you have a nRF52-DK board connected: flash this TAB to your board (using tockloader)

## Running the Integration Tests
Having an nRF52-DK board at hand, integration tests can be run using `./run_hardware_tests.sh`.
The pins P0.03 and P0.04 need to be connected (on a nRF52-DK).
The expected output on the UART console will be as follows.
```
[test-results]
heap_test = "Heap works."
formatting =  works
should_be_one = 1
gpio_works = true
trait_obj_value_usize = 1
trait_obj_value_string = string
callbacks_work = true
all_tests_run = true
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
