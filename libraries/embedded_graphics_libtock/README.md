Embedded Graphics - Libtock
===========================

This crate connects the
[Embedded Graphics library](https://crates.io/crates/embedded-graphics) to
libtock-rs. Specifically, this implements the
[DrawTarget trait](https://docs.rs/embedded-graphics/latest/embedded_graphics/draw_target/trait.DrawTarget.html)
using the Tock `screen` systemcall.
