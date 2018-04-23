use alloc::String;
use core::cell::Cell;
use core::fmt;
use core::fmt::Error;
use core::result::Result;
use result::TockResult;
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

    // TODO: Use &str after relocation is fixed
    pub fn write(&mut self, mut text: String) -> TockResult<()> {
        let num_bytes = text.as_bytes().len();

        let _result = syscalls::allow(DRIVER_NUMBER, allow_nr::SHARE_BUFFER, unsafe {
            text.as_bytes_mut()
        })?;

        let is_written = Cell::new(false);
        let mut is_written_alarm = |_, _, _| is_written.set(true);

        let _subscription = syscalls::subscribe(
            DRIVER_NUMBER,
            subscribe_nr::SET_ALARM,
            &mut is_written_alarm,
        )?;

        unsafe { syscalls::command(DRIVER_NUMBER, command_nr::WRITE, num_bytes, 0) }?;

        syscalls::yieldk_for(|| is_written.get());

        Ok(())
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, string: &str) -> Result<(), Error> {
        self.write(String::from(string)).map_err(|_| Error)
    }
}
