#![no_std]

use core::fmt::Write;
use libtock::adc::AdcBuffer;
use libtock::result::TockResult;
use libtock::syscalls;

#[libtock::main]
/// Reads a 128 byte sample into a buffer and prints the first value to the console.
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let adc_driver = drivers.adc.init_driver()?;
    let mut console = drivers.console.create_console();

    let adc_buffer = AdcBuffer::default();
    let mut temp_buffer = [0; libtock::adc::BUFFER_SIZE];

    let adc_buffer = adc_driver.init_buffer(adc_buffer)?;

    let mut callback = |_, _| {
        adc_buffer.read_bytes(&mut temp_buffer[..]);
        writeln!(console, "First sample in buffer: {}", temp_buffer[0]).unwrap();
    };

    let _subscription = adc_driver.subscribe(&mut callback)?;

    loop {
        adc_driver.sample_continuous_buffered(0, 128)?;
        unsafe { syscalls::raw::yieldk() };
    }
}
