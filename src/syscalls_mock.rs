use core::marker::PhantomData;

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

pub fn unsubscribe(_: usize, _: usize) -> isize {
    unimplemented()
}

pub unsafe fn command(_: usize, _: usize, _: usize, _: usize) -> isize {
    unimplemented()
}

fn unimplemented() -> ! {
    unimplemented!("Unimplemented for tests");
}

pub trait Callback<A> {
    fn driver_number() -> usize;
    fn subscribe_number() -> usize;
}

pub trait ArgumentConverter<CB: ?Sized> {
    fn convert(usize, usize, usize, callback: &mut CB);
}

pub struct Subscription<A, CB: Callback<A>> {
    pub callback: CB,
    pub phantom_data: PhantomData<A>,
}

pub fn subscribe_new<A: ArgumentConverter<CB>, CB: Callback<A>>(
    mut callback: CB,
) -> Subscription<A, CB> {
    extern "C" fn c_callback<A: ArgumentConverter<CB>, CB: Callback<A>>(
        arg0: usize,
        arg1: usize,
        arg2: usize,
        userdata: usize,
    ) {
        let callback = unsafe { &mut *(userdata as *mut CB) };
        A::convert(arg0, arg1, arg2, callback);
    }
    unsafe {
        subscribe(
            CB::driver_number(),
            CB::subscribe_number(),
            c_callback::<A, CB>,
            &mut callback as *mut CB as usize,
        );
    }
    Subscription {
        callback,
        phantom_data: Default::default(),
    }
}

impl<A, CB: Callback<A>> Drop for Subscription<A, CB> {
    fn drop(&mut self) {
        unsubscribe(CB::driver_number(), CB::subscribe_number());
    }
}
