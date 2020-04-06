use crate::executor;
use crate::futures;
use crate::result::TockResult;
use crate::syscalls;
use core::cell::Cell;
use core::convert::From;
use core::fmt;
use core::mem;

pub mod ninedof;

extern "C" fn cb<Reading>(x: usize, y: usize, z: usize, ptr: usize)
where
    Reading: Copy + From<(usize, usize, usize)>,
{
    let res: &Cell<Option<Reading>> = unsafe { mem::transmute(ptr) };
    res.set(Some(From::from((x, y, z))));
}

pub trait Sensor<Reading: Copy + From<(usize, usize, usize)>> {
    fn driver_num(&self) -> usize;

    fn read(&mut self) -> TockResult<Reading> {
        let res: Cell<Option<Reading>> = Cell::new(None);
        let driver_num = self.driver_num();
        syscalls::subscribe_fn(driver_num, 0, cb::<Reading>, unsafe {
            mem::transmute(&res)
        })?;
        syscalls::command(driver_num, 1, 0, 0)?;
        Ok(unsafe { executor::block_on(futures::wait_for_value(|| res.get())) })
    }
}

macro_rules! single_value_sensor {
    ($sensor_name:ident, $type_name:ident, $driver_num:expr) => {
        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        pub struct $type_name {
            value: i32,
        }

        impl From<(usize, usize, usize)> for $type_name {
            fn from(tuple: (usize, usize, usize)) -> $type_name {
                $type_name {
                    value: (tuple.0 as i32),
                }
            }
        }

        impl Into<i32> for $type_name {
            fn into(self) -> i32 {
                self.value
            }
        }

        #[non_exhaustive]
        pub struct $sensor_name;

        impl Sensor<$type_name> for $sensor_name {
            fn driver_num(&self) -> usize {
                $driver_num
            }
        }
    };
}

single_value_sensor!(AmbientLightSensor, AmbientLight, 0x60002);
single_value_sensor!(TemperatureSensor, Temperature, 0x60000);
single_value_sensor!(HumiditySensor, Humidity, 0x60001);

impl fmt::Display for AmbientLight {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} lx", self.value)
    }
}

impl fmt::Display for Humidity {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}.{}%", self.value / 100, self.value % 100)
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}.{}\u{00B0}C", self.value / 100, self.value % 100)
    }
}
