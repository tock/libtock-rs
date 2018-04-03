use core::ptr;
use syscalls;

pub trait SubscribeInfo {
    fn driver_number(&self) -> usize;

    fn subscribe_number(&self) -> usize;
}

pub trait SubscribableCallback {
    fn call_rust(&mut self, arg0: usize, arg1: usize, arg2: usize);
}

pub struct CallbackSubscription<'a, I: SubscribeInfo> {
    #[allow(dead_code)] // Used in drop
    subscribe_info: I,
    _lifetime: &'a (),
}

impl<'a, I: SubscribeInfo> CallbackSubscription<'a, I> {
    pub fn new(subscribe_info: I) -> CallbackSubscription<'a, I> {
        CallbackSubscription {
            subscribe_info,
            _lifetime: &(),
        }
    }
}

impl<'a, I: SubscribeInfo> Drop for CallbackSubscription<'a, I> {
    fn drop(&mut self) {
        unsafe {
            syscalls::subscribe_ptr(
                self.subscribe_info.driver_number(),
                self.subscribe_info.subscribe_number(),
                ptr::null(),
                0,
            );
        }
    }
}
