use callback::CallbackSubscription;
use callback::SubscribableCallback;
use syscalls;

const DRIVER_NUMBER: usize = 0x60000;
const SUBSCRIBE_CALLBACK: usize = 0;
const START_MEASUREMENT: usize = 1;

pub struct TemperatureCallback<CB> {
    callback: CB,
}

impl<CB: FnMut(isize)> SubscribableCallback for TemperatureCallback<CB> {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }

    fn subscribe_number(&self) -> usize {
        SUBSCRIBE_CALLBACK
    }

    fn call_rust(&mut self, arg0: usize, _: usize, _: usize) {
        (self.callback)(arg0 as isize);
    }
}

pub struct TemperatureDriver;

impl TemperatureDriver {
    pub fn start_measurement<CB: FnMut(isize)>(
        callback: CB,
    ) -> Result<CallbackSubscription<TemperatureCallback<CB>>, ()> {
        let (_, subscription) = syscalls::subscribe(TemperatureCallback { callback });
        unsafe {
            syscalls::command(DRIVER_NUMBER, START_MEASUREMENT, 0, 0);
        }
        Ok(subscription)
    }
}
