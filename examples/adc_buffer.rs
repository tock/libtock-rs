#![no_std]

use core::fmt::Write;
use libtock::adc;
use libtock::adc::AdcBuffer;
use libtock::console::Console;
use libtock::syscalls;

/// Reads a 128 byte sample into a buffer and prints the first value to the console.
fn main() {
    let mut console = Console::new();
    let mut adc_buffer = AdcBuffer::new();
    let mut temp_buffer = [0; libtock::adc::BUFFER_SIZE];

    let adc_buffer = libtock::adc::Adc::init_buffer(&mut adc_buffer).unwrap();

    let mut with_callback = adc::with_callback(|_, _| {
        adc_buffer.read_bytes(&mut temp_buffer[..]);
        writeln!(console, "First sample in buffer: {}", temp_buffer[0]).unwrap();
    });

    let adc = with_callback.init().unwrap();

    loop {
        adc.sample_continuous_buffered(0, 128).unwrap();
        syscalls::yieldk();
    }
}
