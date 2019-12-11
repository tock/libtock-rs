use crate::futures;
use crate::result::TockError;
use crate::syscalls;
use core::cell::Cell;
use core::fmt;
use core::fmt::Display;
use core::mem;

const DRIVER_NUMBER: usize = 0x60000;
const SUBSCRIBE_CALLBACK: usize = 0;
const START_MEASUREMENT: usize = 1;

pub async fn measure_temperature() -> Result<Temperature, TockError> {
    let temperature = Cell::<Option<isize>>::new(None);
    let mut callback = |arg1, _, _| temperature.set(Some(arg1 as isize));
    let subscription = syscalls::subscribe(DRIVER_NUMBER, SUBSCRIBE_CALLBACK, &mut callback)?;
    syscalls::command(DRIVER_NUMBER, START_MEASUREMENT, 0, 0)?;
    let temperatur = Temperature {
        centi_celsius: futures::wait_for_value(|| temperature.get()).await,
    };
    mem::drop(subscription);
    Ok(temperatur)
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
    use alloc::string::String;
    use alloc::string::ToString;

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
