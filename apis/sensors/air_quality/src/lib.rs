#![no_std]

use core::cell::Cell;
use libtock_platform::subscribe::OneId;
use libtock_platform::{
    share::scope, share::Handle, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};
use Value::{Tvoc, CO2};

enum Value {
    CO2 = READ_CO2 as isize,
    Tvoc = READ_TVOC as isize,
}

pub struct AirQuality<S: Syscalls>(S);

impl<S: Syscalls> AirQuality<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    /// Register an events listener
    pub fn register_listener<'share, F: Fn(u32)>(
        listener: &'share AirQualityListener<F>,
        subscribe: Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Unregister the events listener
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    /// Initiate a CO2 measurement.
    ///
    /// This function is used both for synchronous and asynchronous readings
    pub fn read_co2() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_CO2, 0, 0).to_result()
    }

    /// Initiate a TVOC measurement.
    ///
    /// This function is used both for synchronous and asynchronous readings
    pub fn read_tvoc() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_TVOC, 0, 0).to_result()
    }

    /// Public wrapper for `read_data_sync` for CO2 synchronous measurement
    pub fn read_co2_sync() -> Result<u32, ErrorCode> {
        Self::read_data_sync(CO2)
    }

    /// Public wrapper for `read_data_sync` for TVOC synchronous measurement
    pub fn read_tvoc_sync() -> Result<u32, ErrorCode> {
        Self::read_data_sync(Tvoc)
    }

    /// Read both CO2 and TVOC values synchronously
    pub fn read_sync() -> Result<(u32, u32), ErrorCode> {
        match (Self::read_data_sync(CO2), Self::read_data_sync(Tvoc)) {
            (Ok(co2_value), Ok(tvoc_value)) => Ok((co2_value, tvoc_value)),
            (Err(co2_error), _) => Err(co2_error),
            (_, Err(tvoc_error)) => Err(tvoc_error),
        }
    }

    /// Initiate a synchronous CO2 or TVOC measurement, based on the `read_type`.
    /// Returns Ok(value) if the operation was successful
    fn read_data_sync(read_type: Value) -> Result<u32, ErrorCode> {
        let data_cell: Cell<Option<u32>> = Cell::new(None);
        let listener = AirQualityListener(|data_val| {
            data_cell.set(Some(data_val));
        });

        scope(|subscribe| {
            Self::register_listener(&listener, subscribe)?;
            match read_type {
                CO2 => {
                    Self::read_co2()?;
                    while data_cell.get().is_none() {
                        S::yield_wait();
                    }

                    match data_cell.get() {
                        None => Err(ErrorCode::Fail),
                        Some(co2_value) => Ok(co2_value),
                    }
                }
                Tvoc => {
                    Self::read_tvoc()?;
                    while data_cell.get().is_none() {
                        S::yield_wait();
                    }

                    match data_cell.get() {
                        None => Err(ErrorCode::Fail),
                        Some(tvoc_value) => Ok(tvoc_value),
                    }
                }
            }
        })
    }
}

pub struct AirQualityListener<F: Fn(u32)>(pub F);
impl<F: Fn(u32)> Upcall<OneId<DRIVER_NUM, 0>> for AirQualityListener<F> {
    fn upcall(&self, data_val: u32, _arg1: u32, _arg2: u32) {
        self.0(data_val)
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60007;

// Command IDs

const EXISTS: u32 = 0;
const READ_CO2: u32 = 2;
const READ_TVOC: u32 = 3;
