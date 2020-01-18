//! Lang item required to make the normal `main` work in applications
//!
//! This is how the `start` lang item works:
//! When `rustc` compiles a binary crate, it creates a `main` function that looks
//! like this:
//!
//! ```
//! #[export_name = "main"]
//! pub extern "C" fn rustc_main(argc: isize, argv: *const *const u8) -> isize {
//!     start(main, argc, argv)
//! }
//! ```
//!
//! Where `start` is this function and `main` is the binary crate's `main`
//! function.
//!
//! The final piece is that the entry point of our program, _start, has to call
//! `rustc_main`. That's covered by the `_start` function in the root of this
//! crate.

use crate::drivers;
use crate::entry_point::TockAllocator;
use crate::leds::LedsDriver;
use crate::result::TockResult;
use crate::timer::Duration;
use crate::timer::ParallelSleepDriver;
use core::alloc::Layout;
use core::executor;
use core::panic::PanicInfo;

#[lang = "start"]
extern "C" fn start<T>(main: fn() -> T, _argc: isize, _argv: *const *const u8)
where
    T: Termination,
{
    main();
}

#[lang = "termination"]
pub trait Termination {}

impl Termination for () {}

impl Termination for crate::result::TockResult<()> {}

#[panic_handler]
unsafe fn panic_handler(_info: &PanicInfo) -> ! {
    // Signal a panic using the LowLevelDebug capsule (if available).
    super::debug::low_level_status_code(1);

    // Flash all LEDs (if available).
    executor::block_on(async {
        let mut drivers = drivers::retrieve_drivers_unsafe();

        let leds_driver = drivers.leds.init_driver();
        let mut timer_driver = drivers.timer.create_timer_driver();
        let timer_driver = timer_driver.activate();

        if let (Ok(leds_driver), Ok(timer_driver)) = (leds_driver, timer_driver) {
            let _ = blink_all_leds(&leds_driver, &timer_driver).await;
        }
        loop {}
    })
}

async fn blink_all_leds(
    leds_driver: &LedsDriver<'_>,
    timer_driver: &ParallelSleepDriver<'_>,
) -> TockResult<()> {
    loop {
        for led in leds_driver.leds() {
            led.on()?;
        }
        timer_driver.sleep(Duration::from_ms(100)).await?;
        for led in leds_driver.leds() {
            led.off()?;
        }
        timer_driver.sleep(Duration::from_ms(100)).await?;
    }
}

#[global_allocator]
static ALLOCATOR: TockAllocator = TockAllocator;

#[alloc_error_handler]
unsafe fn alloc_error_handler(_: Layout) -> ! {
    executor::block_on(async {
        let mut drivers = drivers::retrieve_drivers_unsafe();

        let leds_driver = drivers.leds.init_driver();
        let mut timer_driver = drivers.timer.create_timer_driver();
        let timer_driver = timer_driver.activate();

        if let (Ok(leds_driver), Ok(timer_driver)) = (leds_driver, timer_driver) {
            let _ = cycle_all_leds(&leds_driver, &timer_driver).await;
        }
        loop {}
    })
}

async fn cycle_all_leds(
    leds_driver: &LedsDriver<'_>,
    timer_driver: &ParallelSleepDriver<'_>,
) -> TockResult<()> {
    loop {
        for led in leds_driver.leds() {
            led.on()?;
            timer_driver.sleep(Duration::from_ms(100)).await?;
            led.off()?;
        }
    }
}
