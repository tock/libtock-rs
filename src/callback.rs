use crate::syscalls;
use core::marker::PhantomData;
use core::ptr;

pub trait Consumer<T> {
    fn consume(data: &mut T, arg1: usize, arg2: usize, arg3: usize);
}

pub struct Identity3Consumer;

impl<CB: FnMut(usize, usize, usize)> Consumer<CB> for Identity3Consumer {
    fn consume(data: &mut CB, arg1: usize, arg2: usize, arg3: usize) {
        data(arg1, arg2, arg3);
    }
}

pub struct Identity2Consumer;

impl<CB: FnMut(usize, usize)> Consumer<CB> for Identity2Consumer {
    fn consume(data: &mut CB, arg1: usize, arg2: usize, _: usize) {
        data(arg1, arg2);
    }
}

pub struct Identity1Consumer;

impl<CB: FnMut(usize)> Consumer<CB> for Identity1Consumer {
    fn consume(data: &mut CB, arg1: usize, _: usize, _: usize) {
        data(arg1);
    }
}

pub struct Identity0Consumer;

impl<CB: FnMut()> Consumer<CB> for Identity0Consumer {
    fn consume(data: &mut CB, _: usize, _: usize, _: usize) {
        data();
    }
}

#[must_use = "Subscriptions risk being dropped too early. Drop them manually."]
pub struct CallbackSubscription<'a> {
    driver_number: usize,
    subscribe_number: usize,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> CallbackSubscription<'a> {
    pub(crate) fn new(driver_number: usize, subscribe_number: usize) -> CallbackSubscription<'a> {
        CallbackSubscription {
            driver_number,
            subscribe_number,
            lifetime: PhantomData,
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
