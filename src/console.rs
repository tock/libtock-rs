use alloc::String;
use core::cell::Cell;
use core::fmt;
use core::result::Result;
use shared_memory::ShareableMemory;
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

struct ShareableString {
    string: String,
}

impl ShareableMemory for ShareableString {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }

    fn allow_number(&self) -> usize {
        allow_nr::SHARE_BUFFER
    }

    fn to_bytes(&mut self) -> &mut [u8] {
        unsafe { self.string.as_bytes_mut() }
    }
}

pub struct Console;

impl Console {
    pub fn new() -> Console {
        Console
    }

    // TODO: Use &str after relocation is fixed
    pub fn write(&mut self, text: String) {
        let num_bytes = text.as_bytes().len();

        let text = ShareableString { string: text };

        let (result_code, _shared_string) = syscalls::allow_new(text);
        if result_code < 0 {
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
