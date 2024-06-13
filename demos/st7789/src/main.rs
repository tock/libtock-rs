//! This sample demonstrates displaying text on a ST7789 display
//! using a rust-embedded based crate

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::console::Console;
use libtock::gpio::Gpio;
use libtock::platform::Syscalls;
use libtock::runtime::{set_main, stack_size, TockSyscalls};
use libtock::spi_controller::EmbeddedHalSpi;

use display_interface_spi::SPIInterface;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use mipidsi::{models::ST7789, options::ColorInversion, Builder};

set_main! {main}
stack_size! {0x1000}

// Display
const W: i32 = 240;
const H: i32 = 240;

struct Delay;

impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        Alarm::sleep_for(Milliseconds(ns / (1000 * 1000))).unwrap();
    }
}

fn main() {
    writeln!(Console::writer(), "st7789: example\r").unwrap();

    let mut gpio_dc = Gpio::get_pin(0).unwrap();
    let dc = gpio_dc.make_output().unwrap();

    let mut gpio_reset = Gpio::get_pin(1).unwrap();
    let reset = gpio_reset.make_output().unwrap();

    let di = SPIInterface::new(EmbeddedHalSpi, dc);

    let mut delay = Delay;
    let mut display = Builder::new(ST7789, di)
        .display_size(W as u16, H as u16)
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(reset)
        .init(&mut delay)
        .unwrap();

    // Text
    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    let text = "Hello World ^_^;";
    let text_x = W;
    let text_y = H / 2;

    // Clear the display initially
    display.clear(Rgb565::BLUE).unwrap();

    writeln!(Console::writer(), "Clear complete\r").unwrap();

    // Draw text
    Text::new(text, Point::new(text_x, text_y), text_style)
        .draw(&mut display)
        .unwrap();

    loop {
        TockSyscalls::yield_wait();
    }
}
