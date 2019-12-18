#![no_std]
/**
 * This example shows a repeated timer combined with reading and displaying the current time in
 * clock ticks.
 **/
use core::fmt::Write;
use libtock::console::Console;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    const DELAY_MS: isize = 50;

    let mut console = Console::new();
    let mut previous_ticks = None;

    for i in 0.. {
        let mut timer_with_callback = timer::with_callback(|_, _| {});
        let timer = timer_with_callback.init()?;
        let ticks = timer.get_current_clock()?.num_ticks();
        let frequency = timer.clock_frequency().hz();
        writeln!(
            console,
            "[{}] Waited {} ms. Now is {:#010x} ticks ({} ticks since last time at {} Hz)",
            i,
            i * DELAY_MS,
            ticks,
            previous_ticks.map(|previous| ticks - previous),
            frequency
        )?;
        previous_ticks = Some(ticks);
        timer::sleep(Duration::from_ms(DELAY_MS)).await?;
    }

    Ok(())
}
