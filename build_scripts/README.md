Libtock Build Scripts Support Crate
===================================

This crate provides helpers for building libtock-rs apps.

Usage
-----

There are three general steps to use this crate.

1. This crate should be included as a build dependency in the app's Cargo.toml
   file:

   ```toml
   # Cargo.toml
   ...

   [build-dependencies]
   libtock_build_scripts = { git = "https://github.com/tock/libtock-rs"}
   ```

   This will ensure the crate and the contained linker scripts are available for
   the app build.

2. This crate provides a helper function which can used from the libtock-rs
   app's build.rs file. In the common case, you just call the provided function
   from the build.rs file in your crate's root:

   ```rs
   // build.rs

   fn main() {
       libtock_build_scripts::auto_layout();
   }
   ```

   This will allow cargo to setup linker scripts and paths for the linker when
   your app is built.

3. When calling `cargo build` you need to instruct the build.rs on where in
   memory to compile your app for. This crate supports two mechanisms to do
   this. You can only use one.

   1. Set the `LIBTOCK_PLATFORM` environment variable which specifies the name
      of the linker script in `/layouts` to be used. So for example, if you are
      using the microbit_v2 you might run:

      ```bash
      $ LIBTOCK_PLATFORM=microbit_v2 cargo build --target thumbv7em-none-eabi --release
      ```

   2. Set the `LIBTOCK_LINKER_FLASH` and `LIBTOCK_LINKER_RAM` environment
      variables which specify the starting addresses of flash and RAM memory,
      respectively. This allows you to customize where exactly the compiled app
      must be placed in flash and RAM. For example, to build for common
      Cortex-M4 platforms you might run:

      ```bash
      $ LIBTOCK_LINKER_FLASH=0x00040000 LIBTOCK_LINKER_RAM=0x20008000 cargo build --target thumbv7em-none-eabi --release
      ```
