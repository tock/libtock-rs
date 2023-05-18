//! Fake implementation of the Buzzer API, documented here:
//!
//! Like the real API, `Buzzer` controls a fake buzzer. It provides
//! a function `set_tone` used to immediately call an upcall with a tone set by the buzzer
//! and a function 'set_tone_sync' used to call the upcall when the tone command is received.

use crate::{DriverInfo, DriverShareRef};
use core::time::Duration;
use libtock_platform::{CommandReturn, ErrorCode};
use std::cell::Cell;

// The `upcall_on_command` field is set to Some(value) if an upcall(with value as its argument) should be called when tone command is received,
// or None otherwise. It was needed for testing `tone_sync` library function which simulates a synchronous tone set,
// because it was impossible to schedule an upcall during the `synchronous` tone set in other ways.
pub struct Buzzer {
    busy: Cell<bool>,
    upcall_on_command: [Cell<Option<i32>>; 2],
    share_ref: DriverShareRef,
}

impl Buzzer {
    pub fn new() -> std::rc::Rc<Buzzer> {
        std::rc::Rc::new(Buzzer {
            busy: Cell::new(false),
            upcall_on_command: [Cell::new(None), Cell::new(None)],
            share_ref: Default::default(),
        })
    }

    pub fn is_busy(&self) -> bool {
        self.busy.get()
    }

    pub fn set_tone(&self, freq: i32, duration: Duration) {
        if self.busy.get() {
            self.share_ref
                .schedule_upcall(0, (freq as u32, duration.as_millis() as u32, 0))
                .expect("Unable to schedule upcall");
            self.busy.set(false);
        }
    }

    pub fn set_tone_sync(&self, freq: i32, duration: i32) {
        self.upcall_on_command[0].set(Some(freq));
        self.upcall_on_command[1].set(Some(duration));
    }
}

impl crate::fake::SyscallDriver for Buzzer {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_num: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_num {
            EXISTS => crate::command_return::success(),
            TONE => {
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.busy.set(true);
                if let Some(freq) = self.upcall_on_command[0].take() {
                    if let Some(duration) = self.upcall_on_command[1].take() {
                        self.set_tone(freq, Duration::from_millis(duration as u64));
                    }
                }
                crate::command_return::success()
            }
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x90000;

// Command IDs
const EXISTS: u32 = 0;
const TONE: u32 = 1;
