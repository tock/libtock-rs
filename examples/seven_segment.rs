#![no_std]

use libtock::electronics::ShiftRegister;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Hardware;

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
    let Hardware {
        timer_context,
        gpio_driver,
        ..
    } = libtock::retrieve_hardware()?;
    let mut shift_register = ShiftRegister::new(
        gpio_driver.pin(0)?.open_for_write()?,
        gpio_driver.pin(1)?.open_for_write()?,
        gpio_driver.pin(2)?.open_for_write()?,
    );

    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;

    let mut i = 0;
    loop {
        i = (i + 1) % 11;
        shift_register.write_bits(&number_to_bits(i))?;
        timer_driver.sleep(Duration::from_ms(200)).await?;
    }
}
