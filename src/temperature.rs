use crate::callback::CallbackSubscription;
use crate::callback::SubscribableCallback;
use crate::futures;
use crate::syscalls;
use core::cell::Cell;

const DRIVER_NUMBER: usize = 0x60000;
const SUBSCRIBE_CALLBACK: usize = 0;
const START_MEASUREMENT: usize = 1;

fn with_callback<CB>(callback: CB) -> WithCallback<CB> {
    WithCallback { callback }
}

struct WithCallback<CB> {
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
    fn start_measurement(&mut self) -> Result<CallbackSubscription, isize> {
        let subscription = syscalls::subscribe(DRIVER_NUMBER, SUBSCRIBE_CALLBACK, self)?;
        unsafe { syscalls::command(DRIVER_NUMBER, START_MEASUREMENT, 0, 0) };
        Ok(subscription)
    }
}

pub async fn measure_temperature() -> isize {
    let temperature = Cell::<Option<isize>>::new(None);
    let mut callback = |temp: isize| temperature.set(Some(temp));
    let mut withcallback = with_callback(&mut callback);
    let _subscription = withcallback.start_measurement();
    futures::wait_for_value(|| temperature.get()).await
}
