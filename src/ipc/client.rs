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
        unsafe {
            if syscalls::command(DRIVER_NUM, self.pid, 0) < 0 {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    pub fn subscribe(&mut self, func: Box<FnMut()>) -> Result<(), ()> {
        let func = Box::into_raw(Box::new(func));
        unsafe {
            let res = syscalls::subscribe(DRIVER_NUM, self.pid, Self::cb,
                        func as usize);
            if res < 0 {
                Box::from_raw(func);
                Err(())
            } else {
                Ok(())
            }
        }
    }

    extern fn cb(_: usize, _: usize, _: usize, ptr: usize) {
        unsafe {
            let mut func: Box<Box<FnMut()>> = Box::from_raw(ptr as *mut _);
            func();
            Box::into_raw(func);
        }

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


/*
int ipc_register_client_cb(int svc_id, subscribe_cb callback, void *ud) {
  if (svc_id <= 0) {
    return -1;
  }
  return subscribe(IPC_DRIVER_NUM, svc_id, callback, ud);
}

int ipc_notify_svc(int pid) {
  return command(IPC_DRIVER_NUM, pid, 0);
}

*/
