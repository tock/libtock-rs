use syscalls::{command, subscribe_signed, unsubscribe};

const DRIVER_NUMBER: usize = 0x60000;
const SUBSCRIBE_CALLBACK: usize = 0;
const START_MEASUREMENT: usize = 1;

#[allow(dead_code)]
pub struct TemperatureSensor<CB: TemperatureCallback> {
    callback: CB,
}

pub trait TemperatureCallback {
    fn callback(&mut self, isize);
}
impl<F: FnMut(isize)> TemperatureCallback for F {
    fn callback(&mut self, result: isize) {
        self(result);
    }
}

impl<'a, CB: TemperatureCallback> TemperatureSensor<CB> {
    pub fn start_measurement(callback: CB) -> Result<TemperatureSensor<CB>, &'static str> {
        extern "C" fn cb<CB: TemperatureCallback>(result: isize, _: usize, _: usize, ud: usize) {
            let callback = unsafe { &mut *(ud as *mut CB) };
            callback.callback(result as isize);
        }
        let mut sensor = TemperatureSensor { callback: callback };

        unsafe {
            subscribe_signed(
                DRIVER_NUMBER,
                SUBSCRIBE_CALLBACK,
                cb::<CB>,
                &mut sensor.callback as *mut _ as usize,
            );
            command(DRIVER_NUMBER, START_MEASUREMENT, 0, 0);
        }
        Ok(sensor)
    }
}

impl<CB: TemperatureCallback> Drop for TemperatureSensor<CB> {
    fn drop(&mut self) {
        unsubscribe(DRIVER_NUMBER, SUBSCRIBE_CALLBACK);
    }
}
