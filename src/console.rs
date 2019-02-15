use crate::syscalls;
use core::cell::Cell;
use core::fmt;

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

pub struct Console {
    allow_buffer: [u8; 64],
}

impl Console {
    pub fn new() -> Console {
        Console {
            allow_buffer: [0; 64],
        }
    }

    pub fn write<S: AsRef<[u8]>>(&mut self, text: S) {
        let mut not_written_yet = text.as_ref();
        while !not_written_yet.is_empty() {
            let num_bytes_to_print = self.allow_buffer.len().min(not_written_yet.len());
            self.allow_buffer[..num_bytes_to_print]
                .copy_from_slice(&not_written_yet[..num_bytes_to_print]);
            self.flush(num_bytes_to_print);
            not_written_yet = &not_written_yet[num_bytes_to_print..];
        }
    }

    fn flush(&mut self, num_bytes_to_print: usize) {
        let result = syscalls::allow(
            DRIVER_NUMBER,
            allow_nr::SHARE_BUFFER,
            &mut self.allow_buffer[..num_bytes_to_print],
        );
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
            unsafe { syscalls::command(DRIVER_NUMBER, command_nr::WRITE, num_bytes_to_print, 0) };
        if result_code < 0 {
            return;
        }

        syscalls::yieldk_for(|| is_written.get());
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        self.write(string);
        Ok(())
    }
}
