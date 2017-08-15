use core::cell::Cell;
use core::mem;
use syscalls;
use alloc::heap::Heap;
use alloc::String;
use alloc::boxed::Box;
use alloc::raw_vec::RawVec;
use alloc::allocator::{Alloc, Layout};

const DRIVER_NUM: u32 = 0xff;

pub struct Client {
    pid: u32
}

impl Client {
    pub fn new(pkg_name: String) -> Result<Client, ()> {
        unsafe {
            let res = discover(pkg_name)?;
            Ok(Client {
                pid: res as u32
            })
        }
    }

    pub fn share(&mut self, mut len: usize) -> Result<Box<[u8]>, ()> {
        if len < 3 {
            len = 3;
        }
        unsafe {
            let shared_val = Heap.alloc_zeroed(
                    Layout::from_size_align(1 << len, 1 << len).unwrap());

            match shared_val {
                Ok(v) => {
                    share(self.pid, v, 1 << len)?;
                    Ok(RawVec::from_raw_parts(v, 1 << len).into_box())
                },
                _ => {
                    Err(())
                }
            }
        }
    }

    pub fn notify(&mut self) -> Result<(), ()> {
        let done = Cell::new(false);
        let ptr = &done as *const _ as usize;
        unsafe {
            if syscalls::subscribe(DRIVER_NUM, self.pid, Self::cb, ptr) < 0 {
                return Err(())
            }
            if syscalls::command(DRIVER_NUM, self.pid, 0) < 0 {
                return Err(())
            }
        }
        syscalls::yieldk_for(|| done.get());
        Ok(())
    }

    extern fn cb(_: usize, _: usize, _: usize, ptr: usize) {
        let done: &Cell<bool> = unsafe { mem::transmute(ptr) };
        done.set(true);

    }
}

unsafe fn discover(pkg_name: String) -> Result<isize, ()> {
    let res = syscalls::allow(DRIVER_NUM, 0, pkg_name.as_bytes());
    if res < 0 {
        Err(())
    } else {
        Ok(res)
    }
}

unsafe fn share(pid: u32, base: *mut u8, len: usize) -> Result<(), ()> {
    use core::slice::from_raw_parts;
    let res = syscalls::allow(DRIVER_NUM, pid, from_raw_parts(base, len));
    if res < 0 {
        Err(())
    } else {
        Ok(())
    }
}

