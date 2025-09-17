//! Interface library for using Embedded Graphics with libtock-rs
//!
//! This library implements `embedded_graphics::draw_target::DrawTarget` from
//! the [Embedded Graphics](https://crates.io/crates/embedded-graphics) graphics
//! library using the screen system call driver in Tock.
//!
//! ## Example Usage
//!
//! Using Embedded Graphics to draw a circle on the screen might look like:DrawTarget
//!
//! ```rust
//! use embedded_graphics_libtock::tock_screen::TockMonochrome8BitPage128x64Screen;
//!
//! use embedded_graphics::pixelcolor::BinaryColor;
//! use embedded_graphics::prelude::Point;
//! use embedded_graphics::primitives::{Circle, PrimitiveStyle};
//!
//! let mut screen = TockMonochrome8BitPage128x64Screen::new();
//!
//! let x = 50;
//! let y = 50;
//! let diameter = 40;
//! let _ = Circle::new(Point::new(x as i32, y as i32), diameter)
//!     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
//!     .draw(&mut screen);
//! }
//! let _ = screen.flush();
//! ```

#![no_std]

pub mod tock_screen;
