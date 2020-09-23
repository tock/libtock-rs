use crate::callback::Identity0Consumer;
use crate::executor;
use crate::futures;
use crate::result::TockResult;
use crate::syscalls;
use core::cell::Cell;
use core::fmt;
use core::fmt::Write;
use core::mem;

// Global console
static mut CONSOLE: Option<Console> = None;

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

#[non_exhaustive]
pub struct ConsoleDriver;

impl ConsoleDriver {
    pub fn create_console(self) {
        let console = Console {
            allow_buffer: [0; 64],
        };

        console.set_global_console();
    }
}

pub struct Console {
    allow_buffer: [u8; 64],
}

pub static mut MISSED_PRINT: bool = false;

pub fn get_global_console() -> Option<Console> {
    unsafe {
        if let Some(con) = CONSOLE.take() {
            Some(con)
        } else {
            // If we get here then either the console was never initalised
            // or someone else took it. Set MISSED_PRINT to true so that we
            // can warn the user.
            MISSED_PRINT = true;
            None
        }
    }
}

impl Console {
    pub fn write<S: AsRef<[u8]>>(&mut self, text: S) -> TockResult<()> {
        let mut not_written_yet = text.as_ref();
        while !not_written_yet.is_empty() {
            let num_bytes_to_print = self.allow_buffer.len().min(not_written_yet.len());
            self.allow_buffer[..num_bytes_to_print]
                .copy_from_slice(&not_written_yet[..num_bytes_to_print]);
            self.flush(num_bytes_to_print)?;
            not_written_yet = &not_written_yet[num_bytes_to_print..];
        }
        Ok(())
    }

    fn flush(&mut self, num_bytes_to_print: usize) -> TockResult<()> {
        let shared_memory = syscalls::allow(
            DRIVER_NUMBER,
            allow_nr::SHARE_BUFFER,
            &mut self.allow_buffer[..num_bytes_to_print],
        )?;

        let is_written = Cell::new(false);
        let mut is_written_alarm = || is_written.set(true);
        let subscription = syscalls::subscribe::<Identity0Consumer, _>(
            DRIVER_NUMBER,
            subscribe_nr::SET_ALARM,
            &mut is_written_alarm,
        )?;

        syscalls::command(DRIVER_NUMBER, command_nr::WRITE, num_bytes_to_print, 0)?;

        unsafe { executor::block_on(futures::wait_until(|| is_written.get())) };

        mem::drop(subscription);
        mem::drop(shared_memory);

        Ok(())
    }

    pub fn set_global_console(mut self) {
        unsafe {
            if MISSED_PRINT {
                // Someone tried to print while we were or someone tried
                // to print before initalisation, print a warning here.
                let _ = write!(self, "A print was dropped while the console was locked");
                MISSED_PRINT = false;
            }

            CONSOLE = Some(self);
        }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        self.write(string).map_err(|_| fmt::Error)
    }
}

#[macro_export]
macro_rules! println {
    () => ({
        // Allow an empty println!() to print the location when hit
        println!("")
    });
    ($msg:expr) => ({
        if let Some(mut console) = $crate::console::get_global_console() {
            use core::fmt::Write;
            let _ = writeln!(console, $msg);
            console.set_global_console();
        }
    });
    ($fmt:expr, $($arg:tt)+) => ({
        if let Some(mut console) = $crate::console::get_global_console() {
            use core::fmt::Write;
            let _ = writeln!(console, "{}", format_args!($fmt, $($arg)+));
            console.set_global_console();
        }
    });
}

#[macro_export]
macro_rules! print {
    () => ({
        // Allow an empty print!() to print the location when hit
        print!("")
    });
    ($msg:expr) => ({
        if let Some(mut console) = $crate::console::get_global_console() {
            use core::fmt::Write;
            let _ =  write!(console, $msg);
            console.set_global_console();
        }
    });
    ($fmt:expr, $($arg:tt)+) => ({
        if let Some(mut console) = $crate::console::get_global_console() {
            use core::fmt::Write;
            let _ = write!(console, "{}", format_args!($fmt, $($arg)+));
            console.set_global_console();
        }
    });
}
