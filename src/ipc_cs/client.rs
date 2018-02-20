use alloc::String;
use alloc::allocator::{Alloc, Layout};
use alloc::boxed::Box;
use alloc::heap::Heap;
use alloc::raw_vec::RawVec;
use syscalls::{allow, command, subscribe};

const DRIVER_NUMBER: usize = 0x10000;

mod ipc_commands {
    pub const DISCOVER_SERVICE: usize = 0;
}

pub struct ServerHandle<CB: IPCCallback> {
    pid: isize,
    callback: Option<CB>,
}

pub trait IPCCallback {
    fn callback(&mut self, usize, usize);
}
impl<F: FnMut(usize, usize)> IPCCallback for F {
    fn callback(&mut self, pid: usize, len: usize) {
        self(pid, len);
    }
}

pub fn reserve_shared_buffer() -> Box<[u8]> {
    let shared_val = unsafe {
        Heap.alloc_zeroed(Layout::from_size_align(32, 32).unwrap())
            .unwrap()
    };
    let v = unsafe { RawVec::from_raw_parts(shared_val, 32).into_box() };
    v
}

impl<CB: IPCCallback> ServerHandle<CB> {
    pub fn share(&mut self, shared_buffer: &mut Box<[u8]>, message: &[u8; 32]) {
        shared_buffer.clone_from_slice(message);

        unsafe {
            if allow(DRIVER_NUMBER, self.pid as usize, &*shared_buffer) < 0 {
                panic!()
            };
        }
    }

    pub fn notify(&mut self) {
        unsafe { command(DRIVER_NUMBER, self.pid as usize, 0, 0) };
    }

    pub fn discover_service(name: String) -> Option<ServerHandle<CB>> {
        let pid = unsafe {
            allow(
                DRIVER_NUMBER,
                ipc_commands::DISCOVER_SERVICE,
                &name.as_bytes(),
            )
        };
        if pid >= 0 {
            Some(ServerHandle {
                callback: None,
                pid: pid,
            })
        } else {
            None
        }
    }

    pub fn subscribe_callback(&mut self, mut callback: CB) {
        extern "C" fn cb<CB: IPCCallback>(result: usize, len: usize, _: usize, ud: usize) {
            let callback = unsafe { &mut *(ud as *mut CB) };
            callback.callback(result, len);
        }
        unsafe {
            subscribe(
                DRIVER_NUMBER,
                self.pid as usize,
                cb::<CB>,
                &mut callback as *mut _ as usize,
            );
        }
        self.callback = Some(callback);
    }
}
