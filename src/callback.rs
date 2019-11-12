use crate::syscalls;
use core::marker::PhantomData;
use core::ptr;

pub trait SubscribableCallback {
    fn call_rust(&mut self, arg1: usize, arg2: usize, arg3: usize);
}

impl<F: FnMut(usize, usize, usize)> SubscribableCallback for F {
    fn call_rust(&mut self, arg1: usize, arg2: usize, arg3: usize) {
        self(arg1, arg2, arg3)
    }
}

#[must_use = "Subscriptions risk being dropped too early. Drop them manually."]
pub struct CallbackSubscription<'a> {
    driver_number: usize,
    subscribe_number: usize,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> CallbackSubscription<'a> {
    pub fn new(driver_number: usize, subscribe_number: usize) -> CallbackSubscription<'a> {
        CallbackSubscription {
            driver_number,
            subscribe_number,
            _lifetime: Default::default(),
        }
    }
}

impl<'a> Drop for CallbackSubscription<'a> {
    fn drop(&mut self) {
        unsafe {
            syscalls::raw::subscribe(self.driver_number, self.subscribe_number, ptr::null(), 0);
        }
    }
}
