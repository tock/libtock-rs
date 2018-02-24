use callback::CallbackSubscription;
use callback::SubscribableCallback;
use core::marker::PhantomData;
use syscalls;

const DRIVER_NUMBER: usize = 0x10000;

mod ipc_commands {
    pub const REGISTER_SERVICE: usize = 0;
    pub const NOTIFY_CLIENT: usize = 1;
}

pub struct IpcServerCallback<S, CB> {
    callback: CB,
    phantom_data: PhantomData<S>,
}

impl<S, CB: FnMut(usize, usize, &mut S)> SubscribableCallback for IpcServerCallback<S, CB> {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }

    fn subscribe_number(&self) -> usize {
        ipc_commands::REGISTER_SERVICE
    }

    fn call_rust(&mut self, arg0: usize, arg1: usize, arg2: usize) {
        let data = unsafe { &mut *(arg2 as *mut S) };
        (self.callback)(arg0, arg1, data);
    }
}

pub fn notify_client(pid: usize) {
    unsafe { syscalls::command(DRIVER_NUMBER, pid, ipc_commands::NOTIFY_CLIENT, 0) };
}

pub struct IpcServerDriver;

impl IpcServerDriver {
    pub fn start<S, CB: FnMut(usize, usize, &mut S)>(
        callback: CB,
    ) -> Result<CallbackSubscription<IpcServerCallback<S, CB>>, ()> {
        let (_, subscription) = syscalls::subscribe_new(IpcServerCallback {
            callback,
            phantom_data: Default::default(),
        });
        Ok(subscription)
    }
}
