use crate::callback::CallbackSubscription;
use crate::callback::SubscribableCallback;
use crate::shared_memory::SharedMemory;

pub fn yieldk_for<F: Fn() -> bool>(_: F) {
    unimplemented()
}

pub fn subscribe<CB: SubscribableCallback>(
    _: usize,
    _: usize,
    _: &mut CB,
) -> Result<CallbackSubscription, isize> {
    unimplemented()
}

pub unsafe fn subscribe_ptr(
    _: usize,
    _: usize,
    _: *const unsafe extern "C" fn(usize, usize, usize, usize),
    _: usize,
) -> isize {
    unimplemented()
}

pub unsafe fn command(_: usize, _: usize, _: usize, _: usize) -> isize {
    unimplemented()
}

pub unsafe fn command1_insecure(_: usize, _: usize, _: usize) -> isize {
    unimplemented()
}

pub fn allow(_: usize, _: usize, _: &mut [u8]) -> Result<SharedMemory, isize> {
    unimplemented()
}

pub unsafe fn allow_ptr(_: usize, _: usize, _: *mut u8, _: usize) -> isize {
    unimplemented()
}

pub unsafe fn memop(_: u32, _: usize) -> isize {
    unimplemented()
}

fn unimplemented() -> ! {
    unimplemented!("Unimplemented for tests");
}
