#![no_std]

use libtock::adc::AdcBuffer;
use libtock::println;
use libtock::result::TockResult;
use libtock::syscalls;

#[libtock::main]
/// Reads a 128 byte sample into a buffer and prints the first value to the console.
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let adc_driver = drivers.adc.init_driver()?;
    drivers.console.create_console();

    let mut adc_buffer = AdcBuffer::default();
    let mut temp_buffer = [0; libtock::adc::BUFFER_SIZE];

    let adc_buffer = adc_driver.init_buffer(&mut adc_buffer)?;

    let mut callback = |_, _| {
        adc_buffer.read_bytes(&mut temp_buffer[..]);
        println!("First sample in buffer: {}", temp_buffer[0]);
    };

    let _subscription = adc_driver.subscribe(&mut callback)?;

    loop {
        adc_driver.sample_continuous_buffered(0, 128)?;
        unsafe { syscalls::raw::yieldk() };
    }
}
