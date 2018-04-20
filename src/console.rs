use alloc::String;
use core::cell::Cell;
use core::fmt;
use core::result::Result;
use core::slice;
use syscalls;

const DRIVER_NUMBER: usize = 1;

mod command_nr {
    pub const WRITE: usize = 1;
}

mod subscribe_nr {
    pub const SET_ALARM: usize = 1;
}

mod allow_nr {
    pub const SHARE_BUFFER: usize = 1;
}

pub struct Console;

impl Console {
    pub fn new() -> Console {
        Console
    }

    pub fn write(&mut self, text: String) {
        self.write_bytes(text.as_bytes());
    }

    // TODO: Use this method after relocation is fixed
    pub(crate) fn write_bytes(&mut self, text: &[u8]) {
        let num_bytes = text.len();

        let result = syscalls::allow(DRIVER_NUMBER, allow_nr::SHARE_BUFFER, unsafe {
            slice::from_raw_parts_mut(text.as_ptr() as *mut _, num_bytes)
        });
        if result.is_err() {
            return;
        }

        let is_written = Cell::new(false);
        let mut is_written_alarm = |_, _, _| is_written.set(true);
        let subscription = syscalls::subscribe(
            DRIVER_NUMBER,
            subscribe_nr::SET_ALARM,
            &mut is_written_alarm,
        );
        if subscription.is_err() {
            return;
        }

        let result_code =
            unsafe { syscalls::command(DRIVER_NUMBER, command_nr::WRITE, num_bytes, 0) };
        if result_code < 0 {
            return;
        }

        syscalls::yieldk_for(|| is_written.get());
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        self.write(String::from(string));
        Ok(())
    }
}
