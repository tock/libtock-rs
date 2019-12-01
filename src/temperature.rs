use crate::callback::Identity1Consumer;
use crate::futures;
use crate::result::TockError;
use crate::result::TockResult;
use crate::syscalls;
use core::cell::Cell;
use core::fmt;
use core::fmt::Display;
use core::marker::PhantomData;
use core::mem;

const DRIVER_NUMBER: usize = 0x60000;

mod command_nr {
    pub const IS_DRIVER_AVAILABLE: usize = 0;
    pub const START_MEASUREMENT: usize = 1;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

#[non_exhaustive]
pub struct TemperatureDriverFactory;

impl TemperatureDriverFactory {
    pub fn init_driver(&mut self) -> TockResult<TemperatureDriver> {
        syscalls::command(DRIVER_NUMBER, command_nr::IS_DRIVER_AVAILABLE, 0, 0)?;
        let driver = TemperatureDriver {
            lifetime: PhantomData,
        };
        Ok(driver)
    }
}

pub struct TemperatureDriver<'a> {
    lifetime: PhantomData<&'a ()>,
}

impl<'a> TemperatureDriver<'a> {
    pub async fn measure_temperature(&mut self) -> Result<Temperature, TockError> {
        let temperature = Cell::new(None);
        let mut callback = |centi_celsius| temperature.set(Some(centi_celsius as isize));
        let subscription = syscalls::subscribe::<Identity1Consumer, _>(
            DRIVER_NUMBER,
            subscribe_nr::SUBSCRIBE_CALLBACK,
            &mut callback,
        )?;
        syscalls::command(DRIVER_NUMBER, command_nr::START_MEASUREMENT, 0, 0)?;
        let result = Temperature {
            centi_celsius: futures::wait_for_value(|| temperature.get()).await,
        };
        mem::drop(subscription);
        Ok(result)
    }
}

#[derive(Copy, Clone)]
pub struct Temperature {
    centi_celsius: isize,
}

impl Temperature {
    pub fn in_celsius(self) -> isize {
        self.centi_celsius / 100
    }

    pub fn in_centi_celsius(self) -> isize {
        self.centi_celsius
    }
}

impl Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{:02}°C",
            self.centi_celsius / 100,
            self.centi_celsius.abs() % 100
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn render_temperature() {
        assert_eq!(render_temperature_for(0), "0.00°C");
        assert_eq!(render_temperature_for(5), "0.05°C");
        assert_eq!(render_temperature_for(105), "1.05°C");
        assert_eq!(render_temperature_for(125), "1.25°C");
        assert_eq!(render_temperature_for(1025), "10.25°C");
        assert_eq!(render_temperature_for(-1025), "-10.25°C");
    }

    fn render_temperature_for(centi_celsius: isize) -> String {
        Temperature { centi_celsius }.to_string()
    }
}
