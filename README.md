[![Build Status](https://travis-ci.org/torfmaster/libtock-rs.svg?branch=master)](https://travis-ci.org/torfmaster/libtock-rs)
# libtock-rs
Rust userland library for Tock (WIP)

## Getting Started

This project is nascent and still under heavy development, but first steps:

1. Ensure you have [rustup](https://www.rustup.rs/) installed.

2. Use the `run_example` script to compile and run the example app you want
to use:

    ```bash
    ./run_example.sh blink
    ```

    This should work if you are using the nRF52DK platform. For other platforms,
    you will end up with a TAB file in `target/` that you can program onto your
    Tock board.

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
