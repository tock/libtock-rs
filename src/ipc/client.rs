use alloc::String;
use callback::CallbackSubscription;
use callback::SubscribableCallback;
use shared_memory::SharedMemory;
use syscalls;

const DRIVER_NUMBER: usize = 0x10000;

mod ipc_commands {
    pub const DISCOVER_SERVICE: usize = 0;
}

pub struct ServerHandle {
    pid: usize,
}

pub struct IpcClientCallback<CB> {
    callback: CB,
}

impl<CB> IpcClientCallback<CB> {
    pub fn new(callback: CB) -> Self {
        IpcClientCallback { callback }
    }
}

impl<CB: FnMut(usize, usize)> SubscribableCallback for IpcClientCallback<CB> {
    fn call_rust(&mut self, pid: usize, len: usize, _: usize) {
        (self.callback)(pid, len);
    }
}

pub fn reserve_shared_buffer() -> IPCBuffer {
    IPCBuffer { buffer: [0; 32] }
}

#[repr(align(32))]
pub struct IPCBuffer {
    pub buffer: [u8; 32],
}

impl ServerHandle {
    pub fn share<'a>(&self, shared_buffer: &'a mut IPCBuffer) -> Result<SharedMemory<'a>, isize> {
        syscalls::allow(DRIVER_NUMBER, self.pid as usize, &mut shared_buffer.buffer)
    }

    pub fn notify(&mut self) -> Result<(), isize> {
        let res = unsafe { syscalls::command(DRIVER_NUMBER, self.pid as usize, 0, 0) };
        if res == 0 {
            Ok(())
        } else {
            Err(res)
        }
    }

    pub fn discover_service(mut name: String) -> Option<ServerHandle> {
        let len = name.len();
        let pid = unsafe {
            syscalls::allow_ptr(
                DRIVER_NUMBER,
                ipc_commands::DISCOVER_SERVICE,
                name.as_bytes_mut().as_mut_ptr(),
                len,
            )
        };
        if pid >= 0 {
            Some(ServerHandle { pid: pid as usize })
        } else {
            None
        }
    }

    pub fn subscribe_callback<'a, CB>(
        &self,
        callback: &'a mut IpcClientCallback<CB>,
    ) -> Result<CallbackSubscription<'a>, isize>
    where
        IpcClientCallback<CB>: SubscribableCallback,
    {
        syscalls::subscribe(DRIVER_NUMBER, self.pid, callback)
    }
}
