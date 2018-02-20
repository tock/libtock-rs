use core::marker::PhantomData;
use syscalls;
use syscalls::ArgumentConverter;
use syscalls::Callback;
use syscalls::Subscription;

const DRIVER_NUMBER: usize = 0x10000;

mod ipc_commands {
    pub const REGISTER_SERVICE: usize = 0;
    pub const NOTIFY_CLIENT: usize = 1;
}

pub struct IpcServerConverter<S: Sized> {
    phantom_data: PhantomData<S>,
}
impl<S: Sized, F: FnMut(usize, usize, &mut S)> ArgumentConverter<F> for IpcServerConverter<S> {
    fn convert(arg0: usize, arg1: usize, arg2: usize, callback: &mut F) {
        let data = unsafe { &mut *(arg2 as *mut S) };
        callback(arg0, arg1, data);
    }
}

pub struct IpcServerDriver;

impl<S: Sized, F: FnMut(usize, usize, &mut S)> Callback<IpcServerConverter<S>> for F {
    fn driver_number() -> usize {
        DRIVER_NUMBER
    }

    fn subscribe_number() -> usize {
        ipc_commands::REGISTER_SERVICE
    }
}

pub fn notify_client(pid: usize) {
    unsafe { syscalls::command(DRIVER_NUMBER, pid, ipc_commands::NOTIFY_CLIENT, 0) };
}

impl IpcServerDriver {
    pub fn start<A: ArgumentConverter<CB>, CB: Callback<A>>(
        callback: CB,
    ) -> Result<Subscription<A, CB>, ()> {
        Ok(syscalls::subscribe_new(callback))
    }
}
