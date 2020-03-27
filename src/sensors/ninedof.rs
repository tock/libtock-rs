use crate::executor;
use crate::futures;
use crate::result::TockResult;
use crate::syscalls;
use core::cell::Cell;
use core::fmt;
use core::mem;

const DRIVER_NUM: usize = 0x60004;

#[non_exhaustive]
pub struct NinedofDriver;

#[derive(Copy, Clone, Default, Debug)]
pub struct NinedofReading {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl fmt::Display for NinedofReading {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Default)]
struct CbData {
    res: Cell<NinedofReading>,
    ready: Cell<bool>,
}

impl NinedofDriver {
    pub fn read_acceleration(&mut self) -> TockResult<NinedofReading> {
        let res: CbData = Default::default();
        subscribe(Self::cb, unsafe { mem::transmute(&res) })?;
        start_accel_reading()?;
        unsafe { executor::block_on(futures::wait_until(|| res.ready.get())) };
        Ok(res.res.get())
    }

    pub fn read_magnetometer(&mut self) -> TockResult<NinedofReading> {
        let res: CbData = Default::default();
        subscribe(Self::cb, unsafe { mem::transmute(&res) })?;
        start_magnetometer_reading()?;
        unsafe { executor::block_on(futures::wait_until(|| res.ready.get())) };
        Ok(res.res.get())
    }

    extern "C" fn cb(x: usize, y: usize, z: usize, ptr: usize) {
        let res: &CbData = unsafe { mem::transmute(ptr) };
        res.res.set(NinedofReading {
            x: x as i32,
            y: y as i32,
            z: z as i32,
        });
        res.ready.set(true);
    }
}

pub fn subscribe(cb: extern "C" fn(usize, usize, usize, usize), ud: usize) -> TockResult<()> {
    syscalls::subscribe_fn(DRIVER_NUM, 0, cb, ud)?;
    Ok(())
}

pub fn start_accel_reading() -> TockResult<()> {
    syscalls::command(DRIVER_NUM, 1, 0, 0)?;
    Ok(())
}

pub fn start_magnetometer_reading() -> TockResult<()> {
    syscalls::command(DRIVER_NUM, 100, 0, 0)?;
    Ok(())
}
