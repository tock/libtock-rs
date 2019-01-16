use crate::callback::CallbackSubscription;
use crate::callback::SubscribableCallback;
use crate::syscalls;

const DRIVER_NUMBER: usize = 0x60000;
const SUBSCRIBE_CALLBACK: usize = 0;
const START_MEASUREMENT: usize = 1;

pub fn with_callback<CB>(callback: CB) -> WithCallback<CB> {
    WithCallback { callback }
}

pub struct WithCallback<CB> {
    callback: CB,
}

impl<CB: FnMut(isize)> SubscribableCallback for WithCallback<CB> {
    fn call_rust(&mut self, arg0: usize, _: usize, _: usize) {
        (self.callback)(arg0 as isize);
    }
}

impl<CB> WithCallback<CB>
where
    Self: SubscribableCallback,
{
    pub fn start_measurement(&mut self) -> Result<CallbackSubscription, isize> {
        let subscription = syscalls::subscribe(DRIVER_NUMBER, SUBSCRIBE_CALLBACK, self)?;
        unsafe { syscalls::command(DRIVER_NUMBER, START_MEASUREMENT, 0, 0) };
        Ok(subscription)
    }
}
