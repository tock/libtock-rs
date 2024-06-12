#![no_std]

use core::cell::Cell;
use libtock_platform::{
    share, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

pub struct AmbientLight<S: Syscalls>(S);

impl<S: Syscalls> AmbientLight<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    /// Initiate a light intensity reading.
    pub fn read_intensity() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_INTENSITY, 0, 0).to_result()
    }

    /// Register an events listener
    pub fn register_listener<'share, F: Fn(u32)>(
        listener: &'share IntensityListener<F>,
        subscribe: share::Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Unregister the events listener
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    /// Initiate a synchronous light intensity measurement.
    /// Returns Ok(intensity_value) if the operation was successful
    /// intensity_value is returned in lux
    pub fn read_intensity_sync() -> Result<u32, ErrorCode> {
        let intensity_cell: Cell<Option<u32>> = Cell::new(None);
        let listener = IntensityListener(|intensity_val| {
            intensity_cell.set(Some(intensity_val));
        });

        share::scope(|subscribe| {
            Self::register_listener(&listener, subscribe)?;
            Self::read_intensity()?;
            while intensity_cell.get().is_none() {
                S::yield_wait();
            }

            match intensity_cell.get() {
                None => Err(ErrorCode::Busy),
                Some(intensity_val) => Ok(intensity_val),
            }
        })
    }
}

/// A wrapper around a closure to be registered and called when
/// a luminance reading is done.
///
/// ```ignore
/// let listener = IntensityListener(|intensity_val| {
///     // make use of the intensity value
/// });
/// ```
pub struct IntensityListener<F: Fn(u32)>(pub F);

impl<F: Fn(u32)> Upcall<OneId<DRIVER_NUM, 0>> for IntensityListener<F> {
    fn upcall(&self, intensity: u32, _arg1: u32, _arg2: u32) {
        self.0(intensity)
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60002;

// Command IDs

const EXISTS: u32 = 0;
const READ_INTENSITY: u32 = 1;
