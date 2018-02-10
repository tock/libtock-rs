use syscalls;
use syscalls::ArgumentConverter;
use syscalls::Callback;
use syscalls::Subscription;

const DRIVER_NUMBER: usize = 0x60000;
const SUBSCRIBE_CALLBACK: usize = 0;
const START_MEASUREMENT: usize = 1;

pub struct TemperatureConverter;

impl<F: FnMut(isize)> ArgumentConverter<F> for TemperatureConverter {
    fn convert(arg0: usize, _: usize, _: usize, callback: &mut F) {
        callback(arg0 as isize);
    }
}

impl<F: FnMut(isize)> Callback<TemperatureConverter> for F {
    fn driver_number() -> usize {
        DRIVER_NUMBER
    }

    fn subscribe_number() -> usize {
        SUBSCRIBE_CALLBACK
    }
}

pub struct TemperatureDriver;

impl TemperatureDriver {
    pub fn start_measurement<'a, A: ArgumentConverter<CB>, CB: Callback<A>>(
        callback: CB,
    ) -> Result<Subscription<A, CB>, ()> {
        let sub = syscalls::subscribe_new(callback);
        unsafe {
            syscalls::command(DRIVER_NUMBER, START_MEASUREMENT, 0, 0);
        }
        Ok(sub)
    }
}
