#![no_std]

use core::cell::Cell;

use libtock_platform::{share, DefaultConfig, ErrorCode, Subscribe, Syscalls};

pub struct Proximity<S: Syscalls>(S);

impl<S: Syscalls> Proximity<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
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

    /// Initiate a proximity measurement
    pub fn read() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ, 0, 0).to_result()
    }

    /// Initiate a synchronous proximity measurement.
    /// Returns Ok(proximity_value) if the operation was successful
    /// proximity_value is in [0, 255] range,
    /// where '255' indicates the closest measurable distance and '0' that no object is detected
    pub fn read_sync() -> Result<u8, ErrorCode> {
        let listener: Cell<Option<(u32,)>> = Cell::new(None);
        share::scope(|subscribe| {
            if let Ok(()) = Self::register_listener(&listener, subscribe) {
                if let Ok(()) = Self::read() {
                    S::yield_wait();
                }
            }
        });

        match listener.get() {
            None => Err(ErrorCode::Busy),
            Some(proximity) => Ok(proximity.0 as u8),
        }
    }

    /// Initiate an on_interrupt proximity measurement
    /// The sensor reads values continuously and executes the callback only if
    /// proximity_value < lower or proximity_value > upper
    /// lower - lower interrupt threshold for sensor --> range is [0,255]
    /// upper - upper interrupt threshold for sensor --> range is [0,255]
    /// lower <= upper
    pub fn read_on_interrupt(lower: u8, upper: u8) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_ON_INT, lower as u32, upper as u32).to_result()
    }

    /// Initiate a synchronous on_interrupt proximity measurement.
    /// Returns Ok(proximity_value) if the operation was successful
    /// proximity_value is in [0, 255] range,
    /// where '255' indicates the closest measurable distance and '0' that no object is detected
    /// Returns only when proximity_value < lower or proximity_value > upper
    /// lower - lower interrupt threshold for sensor --> range is [0,255]
    /// upper - upper interrupt threshold for sensor --> range is [0,255]
    /// lower <= upper
    pub fn wait_for_value_between(lower: u8, upper: u8) -> Result<u8, ErrorCode> {
        if lower > upper {
            return Err(ErrorCode::Invalid);
        }
        let listener: Cell<Option<(u32,)>> = Cell::new(None);
        share::scope(|subscribe| {
            if let Ok(()) = Self::register_listener(&listener, subscribe) {
                if let Ok(()) = Self::read_on_interrupt(lower, upper) {
                    while listener.get().is_none() {
                        S::yield_wait();
                    }
                }
            }
        });

        match listener.get() {
            None => Err(ErrorCode::Busy),
            Some(proximity) => Ok(proximity.0 as u8),
        }
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60005;

// Command IDs

const EXISTS: u32 = 0;
const READ: u32 = 1;
const READ_ON_INT: u32 = 2;
