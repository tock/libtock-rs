use core::{fmt,mem};
use core::result::Result;
use syscalls::{self, allow, yieldk};

use alloc::{String, VecDeque};

const DRIVER_NUM: u32 = 0;

pub struct Console {
    queue: VecDeque<String>,
    outstanding: Option<String>
}

impl Console {
    pub fn new() -> Console {
        Console {
            queue: VecDeque::new(),
            outstanding: None
        }
    }

    pub fn write(&mut self, string: String) {
        if self.outstanding.is_none() {
            unsafe {
                putstr_async(&string, Self::cb, self as *const _ as usize);
            }
            self.outstanding = Some(string);
        } else {
            self.queue.push_back(string);
            yieldk();
        }
    }

    extern fn cb(_: usize, _: usize, _: usize, ptr: usize) {
        let console: &mut Console = unsafe {
            mem::transmute(ptr)
        };
        console.outstanding.take();
        if let Some(next) = console.queue.pop_front() {
            unsafe {
                putstr_async(&next, Self::cb, ptr);
            }
            console.outstanding = Some(next);
        }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        self.write(String::from(string));
        Ok(())
    }
}

unsafe fn putstr_async(string: &String, cb: extern fn (usize, usize, usize, usize), ud: usize) -> isize {
  let ret = allow(DRIVER_NUM, 1, string.as_bytes());
  if ret < 0 {
      return ret;
  }

  return syscalls::subscribe(DRIVER_NUM, 1, cb, ud);
}

