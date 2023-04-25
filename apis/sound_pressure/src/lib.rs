#![no_std]

use core::{cell::Cell, convert::TryInto};
use libtock_platform::{share, DefaultConfig, ErrorCode, Subscribe, Syscalls};

pub struct SoundPressure<S: Syscalls>(S);

impl<S: Syscalls> SoundPressure<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    /// Initiate a pressure measurement.
    /// This function is used both for synchronous and asynchronous readings
    pub fn read() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_PRESSURE, 0, 0).to_result()
    }

    /// Register an events listener
    pub fn register_listener<'share>(
        listener: &'share Cell<Option<(u32,)>>,
        subscribe: share::Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Unregister the events listener
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    /// Enable sound pressure measurement
    pub fn enable() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, 2, 0, 0).to_result()
    }

    /// Disable sound pressure measurement
    pub fn disable() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, 3, 0, 0).to_result()
    }

    /// Initiate a synchronous pressure measurement.
    /// Returns Ok(pressure_value) if the operation was successful
    /// pressure_value is between 0 and 255
    pub fn read_sync() -> Result<u8, ErrorCode> {
        let listener: Cell<Option<(u32,)>> = Cell::new(None);
        let err: Result<u8, ErrorCode> = share::scope(|subscribe| {
            Self::register_listener(&listener, subscribe)?;
            {
                Self::read()?;
                {
                    while listener.get() == None {
                        S::yield_wait();
                    }
                    Ok(listener.get().unwrap().0.try_into().unwrap())
                }
            }
        });

        match listener.get() {
            None => err,
            Some(pressure_val) => {
                if !(0..=256).contains(&pressure_val.0) {
                    Err(ErrorCode::Fail)
                } else {
                    pressure_val.0.try_into().map_err(|_e| ErrorCode::Invalid)
                }
            }
        }
    }
}
#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60006;

// Command IDs

const EXISTS: u32 = 0;
const READ_PRESSURE: u32 = 1;
