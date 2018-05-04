[![Build Status](https://travis-ci.org/tock/libtock-rs.svg?branch=master)](https://travis-ci.org/tock/libtock-rs)
# libtock-rs
Rust userland library for Tock (WIP)

## Getting Started

This project is nascent and still under heavy development, but first steps:

1. Ensure you have [rustup](https://www.rustup.rs/) installed.

1. Clone the repository and install its submodules.

    ```bash
    git clone https://github.com/tock/libtock-rs
    git submodule update --init
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

    This should work if you are using the nRF52-DK platform. For other platforms,
    you will end up with a TAB file in `target/tab` that you can program onto your
    Tock board (e.g. with `tockloader install target/tab/blink.tab`).

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
