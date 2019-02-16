[![Build Status](https://travis-ci.org/tock/libtock-rs.svg?branch=master)](https://travis-ci.org/tock/libtock-rs)
# libtock-rs
Rust userland library for Tock (WIP)

Tested with tock a3b36d5872315ff05ef5ad34ed9453b0789218ce.

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

    This should work if you are using the nRF52-DK platform. For other platforms,
    you will end up with a TAB file in `target/tab` that you can program onto your
    Tock board (e.g. with `tockloader install target/tab/blink.tab`).

    If you have a hail board you can flash your device as follows:
     - set the environment variable `hail` to `1`
     - set  `link-arg=-Tnrf52_layout.ld` in `.cargo/config` to `link-arg=-Thail_layout.ld`
     - run `run_example.sh` as above.

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
