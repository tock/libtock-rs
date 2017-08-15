use core::cell::Cell;
use core::{fmt,mem};
use core::result::Result;
use syscalls::{self, allow, yieldk_for};

use alloc::String;

const DRIVER_NUM: u32 = 0;

pub struct Console;

impl Console {
    pub fn new() -> Console {
        Console
    }

    pub fn write(&mut self, string: String) {
        if string.len() <= 0 {
            return;
        }
        let done: Cell<bool> = Cell::new(false);
        unsafe {
            if putstr_async(&string, Self::cb, &done as *const _ as usize) >= 0 {
                yieldk_for(|| done.get())
            } else {
                return
            }
        }
    }

    extern fn cb(_: usize, _: usize, _: usize, ptr: usize) {
        let done: &Cell<bool> = unsafe {
            mem::transmute(ptr)
        };
        done.set(true);
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        self.write(String::from(string));
        Ok(())
    }
}

unsafe fn putstr_async(string: &String, cb: extern fn (usize, usize, usize, usize), ud: usize) -> isize {
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

