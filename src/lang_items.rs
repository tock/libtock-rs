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

use crate::timer::Duration;
use crate::Drivers;
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
        let Drivers {
            led_driver,
            timer_context,
            ..
        } = crate::retrieve_drivers_unsafe();
        let mut driver = timer_context.create_timer_driver();
        let timer_driver = driver.activate().ok();

        loop {
            for led in led_driver.all() {
                let _ = led.on();
            }
            if let Some(ref timer_driver) = timer_driver {
                let _ = timer_driver.sleep(Duration::from_ms(100)).await;
            }
            for led in led_driver.all() {
                let _ = led.off();
            }
            if let Some(ref timer_driver) = timer_driver {
                let _ = timer_driver.sleep(Duration::from_ms(100)).await;
            }
        }
    });
    // Never type is not supported for T in Future
    unreachable!()
}

#[alloc_error_handler]
unsafe fn cycle_leds(_: Layout) -> ! {
    executor::block_on(async {
        let Drivers {
            led_driver,
            timer_context,
            ..
        } = crate::retrieve_drivers_unsafe();
        let mut driver = timer_context.create_timer_driver();
        let timer_driver = driver.activate().ok();

        loop {
            for led in led_driver.all() {
                let _ = led.on();
            }
            if let Some(ref timer_driver) = timer_driver {
                let _ = timer_driver.sleep(Duration::from_ms(100)).await;
            }
            for led in led_driver.all() {
                let _ = led.off();
            }
            if let Some(ref timer_driver) = timer_driver {
                let _ = timer_driver.sleep(Duration::from_ms(100)).await;
            }
        }
    });
    // Never type is not supported for T in Future
    unreachable!()
}
