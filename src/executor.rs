use crate::syscalls;
use core::pin::Pin;
use core::ptr;
use core::task::Context;
use core::task::Poll;
use core::task::RawWaker;
use core::task::RawWakerVTable;
use core::task::Waker;
use futures::Future;

/// # Safety
///
/// [[block_on]] yields whenever a future cannot make any progress at present. Yielding is considered unsafe.
pub unsafe fn block_on<T>(mut future: impl Future<Output = T>) -> T {
    // Contract described in the Rustdoc: "A value, once pinned, must remain pinned forever (...).".
    // IOW calling Pin::new_unchecked is safe as long as no &mut future is leaked after pinning.
    let mut pinned_future = Pin::new_unchecked(&mut future);

    loop {
        match poll(pinned_future.as_mut()) {
            Poll::Pending => syscalls::raw::yieldk(),
            Poll::Ready(value) => {
                return value;
            }
        }
    }
}

fn poll<F: Future>(pinned_future: Pin<&mut F>) -> Poll<F::Output> {
    let waker = unsafe { Waker::from_raw(get_dummy_waker()) };
    let mut context = Context::from_waker(&waker);
    pinned_future.poll(&mut context)
}

// Since Tock OS comes with waking-up functionality built-in, we use dummy wakers that do nothing at all.
fn get_dummy_waker() -> RawWaker {
    fn clone(_x: *const ()) -> RawWaker {
        get_dummy_waker()
    }

    fn do_nothing(_x: *const ()) {}

    // This vtable implements the methods required for managing the lifecycle of the wakers.
    // Our wakers are dummies, so those functions don't do anything.
    static DUMMY_WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(clone, do_nothing, do_nothing, do_nothing);

    // The wakers don't have any implementation, so the instance can simply be null.
    RawWaker::new(ptr::null(), &DUMMY_WAKER_VTABLE)
}
