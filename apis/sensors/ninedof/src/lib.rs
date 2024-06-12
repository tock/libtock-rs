#![no_std]

use core::cell::Cell;
use libtock_platform::{
    share, share::Handle, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

pub struct NineDof<S: Syscalls>(S);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NineDofData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl<S: Syscalls> NineDof<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    /// Initiate a accelerometer measurement.
    /// This function is used both for synchronous and asynchronous readings
    pub fn read_accelerometer() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_ACCELEROMETER, 0, 0).to_result()
    }

    /// Initiate a magnetometer measurement.
    /// This function is used both for synchronous and asynchronous readings
    pub fn read_magnetometer() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_MAGNETOMETER, 0, 0).to_result()
    }

    /// Initiate a gyroscope measurement.
    /// This function is used both for synchronous and asynchronous readings
    pub fn read_gyro() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_GYRO, 0, 0).to_result()
    }

    /// Unregister an events listener
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    /// Register an events listener
    pub fn register_listener<'share, F: Fn(NineDofData)>(
        listener: &'share NineDofListener<F>,
        subscribe: Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Initiate a synchronous accelerometer measurement.
    /// Returns Ok(accelerometer_value) if the operation was successful
    /// Returns Err(ErrorCode) if the operation was unsuccessful
    pub fn read_accelerometer_sync() -> Result<NineDofData, ErrorCode> {
        let data_cell: Cell<Option<NineDofData>> = Cell::new(None);
        let listener = NineDofListener(|data| {
            data_cell.set(Some(data));
        });
        share::scope(|subscribe| {
            Self::register_listener(&listener, subscribe)?;
            Self::read_accelerometer()?;
            while data_cell.get().is_none() {
                S::yield_wait();
            }
            match data_cell.get() {
                None => Err(ErrorCode::Fail),
                Some(data) => Ok(data),
            }
        })
    }

    /// Initiate a synchronous magnetometer measurement.
    /// Returns Ok(data) if the operation was successful
    /// Returns Err(ErrorCode) if the operation was unsuccessful
    pub fn read_magnetometer_sync() -> Result<NineDofData, ErrorCode> {
        let data_cell: Cell<Option<NineDofData>> = Cell::new(None);
        let listener = NineDofListener(|data| {
            data_cell.set(Some(data));
        });
        share::scope(|subscribe| {
            Self::register_listener(&listener, subscribe)?;
            Self::read_magnetometer()?;
            while data_cell.get().is_none() {
                S::yield_wait();
            }
            match data_cell.get() {
                None => Err(ErrorCode::Fail),
                Some(data) => Ok(data),
            }
        })
    }

    /// Initiate a synchronous gyroscope measurement.
    /// Returns Ok(data) as NineDofData if the operation was successful
    /// Returns Err(ErrorCode) if the operation was unsuccessful
    pub fn read_gyroscope_sync() -> Result<NineDofData, ErrorCode> {
        let data_cell: Cell<Option<NineDofData>> = Cell::new(None);
        let listener = NineDofListener(|data| {
            data_cell.set(Some(data));
        });
        share::scope(|subscribe| {
            Self::register_listener(&listener, subscribe)?;
            Self::read_gyro()?;
            while data_cell.get().is_none() {
                S::yield_wait();
            }
            match data_cell.get() {
                None => Err(ErrorCode::Fail),
                Some(data) => Ok(data),
            }
        })
    }

    /// Calculate the magnitude of the accelerometer reading
    /// Returns value of magnitude if the operation was successful
    /// Returns 0.0 if the operation was unsuccessful
    pub fn read_accelerometer_mag() -> f64 {
        let data = Self::read_accelerometer_sync();

        match data {
            Ok(data) => {
                let x = data.x as f64;
                let y = data.y as f64;
                let z = data.z as f64;
                libm::sqrt(x * x + y * y + z * z)
            }
            Err(_) => 0.0,
        }
    }
}

pub struct NineDofListener<F: Fn(NineDofData)>(pub F);

impl<F: Fn(NineDofData)> Upcall<OneId<DRIVER_NUM, 0>> for NineDofListener<F> {
    fn upcall(&self, arg0: u32, arg1: u32, arg2: u32) {
        (self.0)(NineDofData {
            x: arg0 as i32,
            y: arg1 as i32,
            z: arg2 as i32,
        })
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60004;

// Command IDs
const EXISTS: u32 = 0;
const READ_ACCELEROMETER: u32 = 1;
const READ_MAGNETOMETER: u32 = 100;
const READ_GYRO: u32 = 200;
