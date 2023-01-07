#![no_std]

use core::cell::Cell;
use libtock_platform::{share, DefaultConfig, ErrorCode, Subscribe, Syscalls};

pub struct Temperature<S: Syscalls>(S);

impl<S: Syscalls> Temperature<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    /// initiate a temperature measurement used both for syncronous and asyncronous readings
    pub fn read_temperature() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_TEMP, 0, 0).to_result()
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

    /// initiate a syncronous temperature mesurement
    /// Returns Ok(temperature_value) if the operation was successful
    pub fn read_temperature_sync() -> Result<u32, ErrorCode> {
        let temperature_cell: Cell<Option<(u32,)>> = Cell::new(None);

        share::scope(|subscribe| {
            if let Ok(()) = Self::register_listener(&temperature_cell, subscribe) {
                if let Ok(()) = Self::read_temperature() {
                    while temperature_cell.get() == None {
                        S::yield_wait();
                    }
                }
            }
        });

        match temperature_cell.get() {
            None => Err(ErrorCode::Fail),
            Some(temp_val) => Ok(temp_val.0),
        }
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60000;

// Command IDs

const EXISTS: u32 = 0;
const READ_TEMP: u32 = 1;
