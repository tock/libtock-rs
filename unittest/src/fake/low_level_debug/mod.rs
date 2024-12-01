//! Fake implementation of the LowLevelDebug API, documented here:
//! https://github.com/tock/tock/blob/master/doc/syscalls/00008_low_level_debug.md
//!
//! Like the real API, `LowLevelDebug` prints each message it is commanded to
//! print. It also keeps a log of the messages as `Message` instances, which can
//! be retrieved via `take_messages` for use in unit tests.

use crate::DriverInfo;
use libtock_platform::{CommandReturn, ErrorCode};

pub struct LowLevelDebug {
    messages: core::cell::Cell<Vec<Message>>,
}

impl LowLevelDebug {
    pub fn new() -> std::rc::Rc<LowLevelDebug> {
        std::rc::Rc::new(LowLevelDebug {
            messages: Default::default(),
        })
    }

    /// Returns the messages that have been printed, and clears the message
    /// queue.
    pub fn take_messages(&self) -> Vec<Message> {
        self.messages.take()
    }
}

impl crate::fake::SyscallDriver for LowLevelDebug {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM)
    }

    fn command(&self, command_num: u32, argument0: u32, argument1: u32) -> CommandReturn {
        match command_num {
            EXISTS => {}
            PRINT_ALERT_CODE => self.handle_message(Message::AlertCode(argument0)),
            PRINT_1 => self.handle_message(Message::Print1(argument0)),
            PRINT_2 => self.handle_message(Message::Print2(argument0, argument1)),
            _ => return crate::command_return::failure(ErrorCode::NoSupport),
        }
        crate::command_return::success()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Message {
    AlertCode(u32),
    Print1(u32),
    Print2(u32, u32),
}

impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Message::AlertCode(code) => {
                write!(f, "alert code 0x{:x} ({})", code, alert_description(code))
            }
            Message::Print1(arg0) => write!(f, "prints 0x{:x}", arg0),
            Message::Print2(arg0, arg1) => write!(f, "prints 0x{:x} 0x{:x}", arg0, arg1),
        }
    }
}

// -----------------------------------------------------------------------------
// Implementation details below
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests;

const DRIVER_NUM: u32 = 0x8;

// Command numbers
const EXISTS: u32 = 0;
const PRINT_ALERT_CODE: u32 = 1;
const PRINT_1: u32 = 2;
const PRINT_2: u32 = 3;

// Predefined alert codes
const PANIC: u32 = 0x01;
const WRONG_LOCATION: u32 = 0x02;

impl LowLevelDebug {
    fn handle_message(&self, message: Message) {
        // Format the message the same way as the real LowLevelDebug.
        // `libtock_unittest` doesn't support multiple processes, so we pretend
        // this is the first process (number 0).
        println!("LowLevelDebug: App 0x0 {}", message);
        let mut messages = self.messages.take();
        messages.push(message);
        self.messages.set(messages);
    }
}

// Returns a description of a predefined alert code.
fn alert_description(code: u32) -> &'static str {
    match code {
        PANIC => "panic",
        WRONG_LOCATION => "wrong location",
        _ => "unknown",
    }
}
