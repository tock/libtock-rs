#![no_std]

use core::cell::Cell;

use libtock_platform::{share, subscribe::AnyId, DefaultConfig, ErrorCode, Syscalls, Upcall};

/// The Buttonss driver
///
/// # Example
/// ```ignore
/// use libtock2::Buttons;
///
/// // Read button state
/// Buttons::is_pressed(0);
///
/// // Register for events
/// Buttons::register_for_events(
///     |button, state| { // print state of button },
///     || {
///         // execute something while registered to receive events
///     }
/// );
/// ```
pub struct Buttons<S: Syscalls>(S);

#[derive(PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
    Unknown,
}

impl From<u32> for ButtonState {
    fn from(original: u32) -> ButtonState {
        match original {
            0 => ButtonState::Released,
            1 => ButtonState::Pressed,
            _ => ButtonState::Unknown,
        }
    }
}

struct ButtonUpcall<F: Fn(u32, ButtonState)> {
    event: F,
}

impl<F: Fn(u32, ButtonState)> Upcall<AnyId> for ButtonUpcall<F> {
    fn upcall(&self, button: u32, state: u32, _arg2: u32) {
        (self.event)(button, state.into());
    }
}

impl<S: Syscalls> Buttons<S> {
    /// Run a check against the buttons capsule to ensure it is present.
    ///
    /// Returns `Some(number_of_buttons)` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    pub fn count() -> Option<u32> {
        S::command(DRIVER_ID, BUTTONS_COUNT, 0, 0).get_success_u32()
    }

    pub fn read(button: u32) -> Result<ButtonState, ErrorCode> {
        let command_return = S::command(DRIVER_ID, BUTTONS_READ, button, 0);
        if let Some(error) = command_return.get_failure() {
            Err(error)
        } else {
            match command_return.get_success_u32() {
                Some(value) => Ok(value.into()),
                None => {
                    unreachable!()
                }
            }
        }
    }

    pub fn is_pressed(button: u32) -> bool {
        Self::read(button)
            .map(|state| state == ButtonState::Pressed)
            .unwrap_or(false)
    }

    pub fn is_released(button: u32) -> bool {
        Self::read(button)
            .map(|state| state == ButtonState::Released)
            .unwrap_or(false)
    }

    pub fn enable_interrupts(button: u32) -> Result<(), ErrorCode> {
        if let Some(error) =
            S::command(DRIVER_ID, BUTTONS_ENABLE_INTERRUPTS, button, 0).get_failure()
        {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn disable_interrupts(button: u32) -> Result<(), ErrorCode> {
        if let Some(error) =
            S::command(DRIVER_ID, BUTTONS_DISABLE_INTERRUPTS, button, 0).get_failure()
        {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub async fn wait_for_pressed(button: u32) -> Result<(), ErrorCode> {
        if Self::is_pressed(button) {
            Ok(())
        } else {
            Self::wait_switch_pressed(button).await
        }
    }

    pub async fn wait_for_released(button: u32) -> Result<(), ErrorCode> {
        if Self::is_released(button) {
            Ok(())
        } else {
            Self::wait_switch_released(button).await
        }
    }

    pub async fn wait_switch_pressed(button: u32) -> Result<(), ErrorCode> {
        let called = Cell::<Option<(u32, u32)>>::new(None);
        share::scope(|subscribe| {
            Self::enable_interrupts(button)?;
            S::subscribe::<_, _, DefaultConfig, DRIVER_ID, 0>(subscribe, &called)?;
            // futures::wait_until(|| {
            //     if let Some((pressed_button, 1)) = called.get() {
            //         button == pressed_button
            //     } else {
            //         false
            //     }
            // })
            // .await;
            let _ = Self::disable_interrupts(button);
            Ok(())
        })
    }

    pub async fn wait_switch_released(button: u32) -> Result<(), ErrorCode> {
        let called = Cell::<Option<(u32, u32)>>::new(None);
        share::scope(|subscribe| {
            Self::enable_interrupts(button)?;
            S::subscribe::<_, _, DefaultConfig, DRIVER_ID, 0>(subscribe, &called)?;
            // futures::wait_until(|| {
            //     if let Some((released_button, 1)) = called.get() {
            //         button == released_button
            //     } else {
            //         false
            //     }
            // })
            // .await;
            let _ = Self::disable_interrupts(button);
            Ok(())
        })
    }

    pub fn register_for_events<Fe, Fr>(event: Fe, run: Fr) -> Result<(), ErrorCode>
    where
        Fe: Fn(u32, ButtonState),
        Fr: FnOnce(),
    {
        let called = ButtonUpcall { event };
        share::scope(|subscribe| {
            S::subscribe::<_, _, DefaultConfig, DRIVER_ID, 0>(subscribe, &called)?;
            run();
            Ok(())
        })
    }
}

const DRIVER_ID: u32 = 2;

// Command IDs
const BUTTONS_COUNT: u32 = 0;

const BUTTONS_ENABLE_INTERRUPTS: u32 = 1;
const BUTTONS_DISABLE_INTERRUPTS: u32 = 2;

const BUTTONS_READ: u32 = 3;

#[cfg(test)]
mod tests;
