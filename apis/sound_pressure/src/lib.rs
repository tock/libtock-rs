#![no_std]

use core::cell::Cell;
use libtock_platform::{
    share, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

pub struct SoundPressure<S: Syscalls>(S);

impl<S: Syscalls> SoundPressure<S> {
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    pub fn read_pressure() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_PRESSURE, 0, 0).to_result()
    }

    pub fn register_listener<'share, F: Fn(i32)>(
        listener: &'share SoundPressureListener<F>,
        subscribe: share::Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    pub fn sound_pressure_enabled() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, 2, 0, 0).to_result()
    }

    pub fn sound_pressure_disabled() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, 3, 0, 0).to_result()
    }

    pub fn read_pressure_sync() -> Result<i32, ErrorCode> {
        let pressure_cell: Cell<Option<i32>> = Cell::new(None);
        let listener = SoundPressureListener(|pressure_val| {
            pressure_cell.set(Some(pressure_val));
        });
        share::scope(|subscribe| {
            if let Ok(()) = Self::register_listener(&listener, subscribe) {
                if let Ok(()) = Self::read_pressure() {
                    while pressure_cell.get() == None {
                        S::yield_wait();
                    }
                }
            }
        });

        match pressure_cell.get() {
            None => Err(ErrorCode::Busy),
            Some(pressure_val) => {
                if pressure_val < 0 || pressure_val > 256 {
                    Err(ErrorCode::Fail)
                } else {
                    Ok(pressure_val)
                }
            }
        }
    }
}

pub struct SoundPressureListener<F: Fn(i32)>(pub F);
impl<F: Fn(i32)> Upcall<OneId<DRIVER_NUM, 0>> for SoundPressureListener<F> {
    fn upcall(&self, pressure_val: u32, _arg1: u32, _arg2: u32) {
        (self.0)(pressure_val as i32);
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
