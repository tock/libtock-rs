#![no_std]

use libtock::alarm::Duration;
use libtock::electronics::ShiftRegister;
use libtock::result::TockResult;

libtock_core::stack_size! {0x800}

fn number_to_bits(n: u8) -> [bool; 8] {
    match n {
        1 => [false, false, false, true, false, true, false, false],
        2 => [true, false, true, true, false, false, true, true],
        3 => [true, false, true, true, false, true, true, false],
        4 => [true, true, false, true, false, true, false, false],
        5 => [true, true, true, false, false, true, true, false],
        6 => [true, true, true, false, false, true, true, true],
        7 => [false, false, true, true, false, true, false, false],
        8 => [true, true, true, true, false, true, true, true],
        9 => [true, true, true, true, false, true, true, false],
        0 => [false, true, true, true, false, true, true, true],
        _ => [false, false, false, false, true, false, false, false],
    }
}

// Example works on a shift register on P0.03, P0.04, P0.28
#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let mut gpio_driver = drivers.gpio.init_driver()?;
    let mut gpios = gpio_driver.gpios();
    let mut gpio0 = gpios.next().unwrap();
    let gpio0 = gpio0.enable_output()?;
    let mut gpio1 = gpios.next().unwrap();
    let gpio1 = gpio1.enable_output()?;
    let mut gpio2 = gpios.next().unwrap();
    let gpio2 = gpio2.enable_output()?;
    let mut shift_register = ShiftRegister::new(&gpio0, &gpio1, &gpio2);

    let mut driver = drivers.alarm.create_timer_driver();
    let timer_driver = driver.activate()?;

    let mut i = 0;
    loop {
        i = (i + 1) % 11;
        shift_register.write_bits(&number_to_bits(i))?;
        timer_driver.sleep(Duration::from_ms(200)).await?;
    }
}
