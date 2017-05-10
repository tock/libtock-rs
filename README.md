# libtock-rs
Rust userland library for Tock (WIP)

## Getting Started

This project is nascent and still under heavy development, but first steps:

1. Get a copy of the latest nightly, in this repo's root:

    `rustup override set nightly`

2. Need to grab a copy of the rust sources:

    `rustup component add rust-src`

3. Now you should be able to build with:

    `xargo build --target thumbv7em-tock-eabi`
