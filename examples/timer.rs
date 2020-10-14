#![no_std]
use libtock::println;
/**
 * This example shows a repeated timer combined with reading and displaying the current time in
 * clock ticks.
 **/
use libtock::result::TockResult;
use libtock::timer::DriverContext;
use libtock::timer::Duration;

const DELAY_MS: usize = 500;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    drivers.console.create_console();

    let mut previous_ticks = None;

    for i in 0.. {
        print_now(&mut drivers.timer, &mut previous_ticks, i)?;
        let mut timer_driver = drivers.timer.create_timer_driver();
        let timer_driver = timer_driver.activate()?;

        timer_driver.sleep(Duration::from_ms(DELAY_MS)).await?;
    }

    Ok(())
}

fn print_now(
    timer_context: &mut DriverContext,
    previous_ticks: &mut Option<isize>,
    i: usize,
) -> TockResult<()> {
    let mut timer_with_callback = timer_context.with_callback(|_, _| {});
    let timer = timer_with_callback.init()?;
    let current_clock = timer.get_current_clock()?;
    let ticks = current_clock.num_ticks();
    let frequency = timer.clock_frequency().hz();
    println!(
        "[{}] Waited roughly {}. Now is {} = {:#010x} ticks ({:?} ticks since last time at {} Hz)",
        i,
        PrettyTime::from_ms(i * DELAY_MS),
        PrettyTime::from_ms(current_clock.ms_f64() as usize),
        ticks,
        previous_ticks.map(|previous| ticks - previous),
        frequency
    );
    *previous_ticks = Some(ticks);
    Ok(())
}

struct PrettyTime {
    mins: usize,
    secs: usize,
    ms: usize,
}

impl PrettyTime {
    fn from_ms(ms: usize) -> PrettyTime {
        PrettyTime {
            ms: ms % 1000,
            secs: (ms / 1000) % 60,
            mins: ms / (60 * 1000),
        }
    }
}

impl core::fmt::Display for PrettyTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.mins != 0 {
            write!(f, "{}m", self.mins)?
        }
        write!(f, "{}.{:03}s", self.secs, self.ms)
    }
}
