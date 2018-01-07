# libtock-rs
Rust userland library for Tock (WIP)

## Getting Started

This project is nascent and still under heavy development, but first steps:

1. Ensure you have a working toolchain available:

    `rustup install nightly-2017-12-16`

2. Get a copy of this toolchain, in this repo's root:

    `rustup override set nightly-2017-12-16`

3. Need to grab a copy of the rust sources:

    `rustup component add rust-src`

4. Now you should be able to build with:

    `xargo build --target thumbv7em-tock-eabi`


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
