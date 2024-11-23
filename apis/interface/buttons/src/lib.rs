#![no_std]

use libtock_platform::{
    share::Handle, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

/// The Buttons driver
///
/// # Example
/// ```ignore
/// use libtock::Buttons;
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl From<u32> for ButtonState {
    fn from(value: u32) -> ButtonState {
        match value {
            0 => ButtonState::Released,
            _ => ButtonState::Pressed,
        }
    }
}

impl<S: Syscalls> Buttons<S> {
    /// Run a check against the buttons capsule to ensure it is present.
    ///
    /// Returns `Ok(number_of_buttons)` if the driver was present. This does not necessarily mean
    /// that the driver is working.
    pub fn count() -> Result<u32, ErrorCode> {
        S::command(DRIVER_NUM, BUTTONS_COUNT, 0, 0).to_result()
    }

    /// Read the state of a button
    pub fn read(button: u32) -> Result<ButtonState, ErrorCode> {
        let button_state: u32 = S::command(DRIVER_NUM, BUTTONS_READ, button, 0).to_result()?;
        Ok(button_state.into())
    }

    /// Returns `true` if a button is pressed
    ///
    /// This function returns `false` if:
    /// - the button is released
    /// - the button number is invalid
    /// - there is an error
    pub fn is_pressed(button: u32) -> bool {
        Self::read(button)
            .map(|state| state == ButtonState::Pressed)
            .unwrap_or(false)
    }

    /// Returns `true` if a button is released
    ///
    /// This function returns `false` if:
    /// - the button is pressed
    /// - the button number is invalid
    /// - there is an error
    pub fn is_released(button: u32) -> bool {
        Self::read(button)
            .map(|state| state == ButtonState::Released)
            .unwrap_or(false)
    }

    /// Enable events (interrupts) for a button
    pub fn enable_interrupts(button: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, BUTTONS_ENABLE_INTERRUPTS, button, 0).to_result()
    }

    /// Disable events (interrupts) for a button
    pub fn disable_interrupts(button: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, BUTTONS_DISABLE_INTERRUPTS, button, 0).to_result()
    }

    /// Register an events listener
    ///
    /// There can be only one single listener registered at a time.
    /// Each time this function is used, it will replace the
    /// previously registered listener.
    pub fn register_listener<'share, F: Fn(u32, ButtonState)>(
        listener: &'share ButtonListener<F>,
        subscribe: Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Unregister the events listener
    ///
    /// This function may be used even if there was no
    /// previously registered listener.
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }
}

/// A wrapper around a closure to be registered and called when
/// a button event occurs.
///
/// ```ignore
/// let listener = ButtonListener(|button, state| {
///     // make use of the button's state
/// });
/// ```
pub struct ButtonListener<F: Fn(u32, ButtonState)>(pub F);

impl<F: Fn(u32, ButtonState)> Upcall<OneId<DRIVER_NUM, 0>> for ButtonListener<F> {
    fn upcall(&self, button_index: u32, state: u32, _arg2: u32) {
        self.0(button_index, state.into())
    }
}
#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x3;

// Command IDs
const BUTTONS_COUNT: u32 = 0;

const BUTTONS_ENABLE_INTERRUPTS: u32 = 1;
const BUTTONS_DISABLE_INTERRUPTS: u32 = 2;

const BUTTONS_READ: u32 = 3;
