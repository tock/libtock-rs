use crate::syscalls;
use core::ptr;

pub trait SubscribableCallback {
    fn call_rust(&mut self, arg0: usize, arg1: usize, arg2: usize);
}

impl<F: FnMut(usize, usize, usize)> SubscribableCallback for F {
    fn call_rust(&mut self, arg0: usize, arg1: usize, arg2: usize) {
        self(arg0, arg1, arg2)
    }
}

pub struct CallbackSubscription<'a> {
    driver_number: usize,
    subscribe_number: usize,
    _lifetime: &'a (),
}

impl<'a> CallbackSubscription<'a> {
    pub fn new(driver_number: usize, subscribe_number: usize) -> CallbackSubscription<'a> {
        CallbackSubscription {
            driver_number,
            subscribe_number,
            _lifetime: &(),
        }
    }
}

impl<'a> Drop for CallbackSubscription<'a> {
    fn drop(&mut self) {
        unsafe {
            syscalls::subscribe_ptr(self.driver_number, self.subscribe_number, ptr::null(), 0);
        }
    }
}
