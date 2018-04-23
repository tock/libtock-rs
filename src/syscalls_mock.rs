use callback::CallbackSubscription;
use callback::SubscribableCallback;
use result::TockResult;
use shared_memory::SharedMemory;

pub fn yieldk_for<F: Fn() -> bool>(_: F) {
    unimplemented()
}

pub fn subscribe<CB: SubscribableCallback>(
    _: usize,
    _: usize,
    _: &mut CB,
) -> TockResult<CallbackSubscription> {
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

pub unsafe fn command(_: usize, _: usize, _: usize, _: usize) -> TockResult<usize> {
    unimplemented()
}

pub fn allow(_: usize, _: usize, _: &mut [u8]) -> TockResult<SharedMemory> {
    unimplemented()
}

pub unsafe fn allow_ptr(_: usize, _: usize, _: *mut u8, _: usize) -> isize {
    unimplemented()
}

fn unimplemented() -> ! {
    unimplemented!("Unimplemented for tests");
}
