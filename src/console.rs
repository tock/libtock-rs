use alloc::String;
use core::{fmt, mem};
use core::cell::Cell;
use core::result::Result;
use syscalls::{self, allow, yieldk_for};

const DRIVER_NUM: u32 = 1;

pub struct Console;

impl Console {
    pub fn new() -> Console {
        Console
    }

    // TODO: Accept borrowed strings (e.g. &str).
    // For this, the borrowed referencne must be accessible by the capsule. This is not the case for string literals.
    pub fn write(&mut self, string: String) {
        if string.len() <= 0 {
            return;
        }
        let done: Cell<bool> = Cell::new(false);
        unsafe {
            if putstr_async(&string, Self::cb, &done as *const _ as usize) >= 0 {
                yieldk_for(|| done.get())
            } else {
                return;
            }
        }
    }

    extern "C" fn cb(_: usize, _: usize, _: usize, ptr: usize) {
        let done: &Cell<bool> = unsafe { mem::transmute(ptr) };
        done.set(true);
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        self.write(String::from(string));
        Ok(())
    }
}

// TODO: Should this function be unsafe on its own?
unsafe fn putstr_async(
    string: &String,
    cb: extern "C" fn(usize, usize, usize, usize),
    ud: usize,
) -> isize {
    let mut ret = allow(DRIVER_NUM, 1, string.as_bytes());
    if ret < 0 {
        return ret;
    }

    ret = syscalls::subscribe(DRIVER_NUM, 1, cb, ud);
    if ret < 0 {
        return ret;
    }

    ret = syscalls::command(DRIVER_NUM, 1, string.len() as isize);
    if ret < 0 {
        return ret;
    }
    ret
}
