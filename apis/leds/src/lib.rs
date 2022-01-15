#![no_std]

use libtock_platform::Syscalls;

/// The LEDs driver
///
/// # Example
/// ```ignore
/// use libtock2::Leds;
///
/// // Turn on led 0
/// Leds::on(0);
/// ```
pub struct Leds<S: Syscalls>(S);

impl<S: Syscalls> Leds<S> {
    /// Run a check against the leds capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn driver_check() -> bool {
        S::command(DRIVER_ID, DRIVER_CHECK, 0, 0).is_success_u32()
    }

    #[inline(always)]
    pub fn count() -> usize {
        S::command(DRIVER_ID, LEDS_COUNT, 0, 0)
            .get_success_u32()
            .unwrap_or_default() as usize
    }

    #[inline(always)]
    pub fn on(led: usize) {
        S::command(DRIVER_ID, LED_ON, led as u32, 0);
    }

    #[inline(always)]
    pub fn off(led: usize) {
        S::command(DRIVER_ID, LED_OFF, led as u32, 0);
    }

    #[inline(always)]
    pub fn toggle(led: usize) {
        S::command(DRIVER_ID, LED_TOGGLE, led as u32, 0);
    }
}

const DRIVER_ID: u32 = 2;

// Command IDs
const DRIVER_CHECK: u32 = 0;
const LEDS_COUNT: u32 = 0;
const LED_ON: u32 = 1;
const LED_OFF: u32 = 2;
const LED_TOGGLE: u32 = 3;

#[cfg(test)]
mod tests;
