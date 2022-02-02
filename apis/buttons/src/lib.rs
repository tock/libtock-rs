#![no_std]

use libtock_platform::{
    share::Handle, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

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
///
/// let listener = ButtonListener(|button, state| {
///     // make use of the button's state
/// });
///
/// share::scope(|subscribe| {
///     if let Ok(()) = Buttons::register_listener(&listener, subscribe) {
///         // yield
///     }
/// });
/// ```
pub struct Buttons<S: Syscalls>(S);

#[derive(Debug, PartialEq)]
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

impl<S: Syscalls> Buttons<S> {
    /// Run a check against the buttons capsule to ensure it is present.
    ///
    /// Returns `Some(number_of_buttons)` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    pub fn count() -> Result<u32, ErrorCode> {
        Ok(S::command(DRIVER_ID, BUTTONS_COUNT, 0, 0).to_result()?)
    }

    pub fn read(button: u32) -> Result<ButtonState, ErrorCode> {
        let button_state: u32 = S::command(DRIVER_ID, BUTTONS_READ, button, 0).to_result()?;
        Ok(button_state.into())
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
        S::command(DRIVER_ID, BUTTONS_ENABLE_INTERRUPTS, button, 0).to_result()
    }

    pub fn disable_interrupts(button: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_ID, BUTTONS_DISABLE_INTERRUPTS, button, 0).to_result()
    }

    pub fn register_listener<'share, F: Fn(u32, ButtonState)>(
        listener: &'share ButtonListener<F>,
        subscribe: Handle<Subscribe<'share, S, DRIVER_ID, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_ID, 0>(subscribe, listener)
    }

    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_ID, 0)
    }
}

pub struct ButtonListener<F: Fn(u32, ButtonState)>(F);

impl<F: Fn(u32, ButtonState)> Upcall<OneId<DRIVER_ID, 0>> for ButtonListener<F> {
    fn upcall(&self, button_index: u32, state: u32, _arg2: u32) {
        self.0(button_index, state.into())
    }
}

const DRIVER_ID: u32 = 3;

// Command IDs
const BUTTONS_COUNT: u32 = 0;

const BUTTONS_ENABLE_INTERRUPTS: u32 = 1;
const BUTTONS_DISABLE_INTERRUPTS: u32 = 2;

const BUTTONS_READ: u32 = 3;

#[cfg(test)]
mod tests;
