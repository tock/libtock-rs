use callback::CallbackSubscription;
use callback::SubscribableCallback;
use shared_memory::ShareableMemory;
use shared_memory::SharedMemory;

pub fn yieldk_for<F: Fn() -> bool>(_: F) {
    unimplemented()
}

pub unsafe fn allow(_: usize, _: usize, _: &[u8]) -> isize {
    unimplemented()
}

pub unsafe fn allow16(_: usize, _: usize, _: &[u16]) -> isize {
    unimplemented()
}

pub unsafe fn subscribe(
    _: usize,
    _: usize,
    _: unsafe extern "C" fn(usize, usize, usize, usize),
    _: usize,
) -> isize {
    unimplemented()
}

pub unsafe fn command(_: usize, _: usize, _: usize, _: usize) -> isize {
    unimplemented()
}

pub fn subscribe_new<CB: SubscribableCallback>(_: CB) -> (isize, CallbackSubscription<CB>) {
    unimplemented()
}

pub fn allow_new<SM: ShareableMemory>(_: SM) -> (isize, SharedMemory<SM>) {
    unimplemented()
}

fn unimplemented() -> ! {
    unimplemented!("Unimplemented for tests");
}
