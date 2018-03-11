use alloc::String;
use alloc::allocator::{Alloc, Layout};
use alloc::boxed::Box;
use alloc::heap::Heap;
use alloc::raw_vec::RawVec;
use callback::CallbackSubscription;
use callback::SubscribableCallback;
use result::TockResult;
use syscalls;

const DRIVER_NUMBER: usize = 0x10000;

mod ipc_commands {
    pub const DISCOVER_SERVICE: usize = 0;
}

pub struct ServerHandle {
    pid: isize,
}

pub struct IpcClientCallback<CB> {
    callback: CB,
    pid: isize,
}

impl<CB: FnMut(usize, usize)> SubscribableCallback for IpcClientCallback<CB> {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }

    fn subscribe_number(&self) -> usize {
        self.pid as usize
    }

    fn call_rust(&mut self, pid: usize, len: usize, _: usize) {
        (self.callback)(pid, len);
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

impl ServerHandle {
    pub fn share(&mut self, shared_buffer: &mut Box<[u8]>, message: &[u8; 32]) {
        shared_buffer.clone_from_slice(message);

        unsafe {
            if syscalls::allow(DRIVER_NUMBER, self.pid as usize, &*shared_buffer) < 0 {
                panic!()
            };
        }
    }

    pub fn notify(&mut self) {
        unsafe { syscalls::command(DRIVER_NUMBER, self.pid as usize, 0, 0) };
    }

    pub fn discover_service(name: String) -> Option<ServerHandle> {
        let pid = unsafe {
            syscalls::allow(
                DRIVER_NUMBER,
                ipc_commands::DISCOVER_SERVICE,
                &name.as_bytes(),
            )
        };
        if pid >= 0 {
            Some(ServerHandle { pid: pid })
        } else {
            None
        }
    }

    pub fn subscribe_callback<CB: FnMut(usize, usize)>(
        &self,
        callback: CB,
    ) -> TockResult<CallbackSubscription<IpcClientCallback<CB>>, isize> {
        let (_, handle) = syscalls::subscribe_new(IpcClientCallback {
            callback: callback,
            pid: self.pid,
        });
        Ok(handle)
    }
}
