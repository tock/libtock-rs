#![no_std]

use core::cell::Cell;
use libtock_platform::{
    share::scope, share::Handle, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

pub struct AirQuality<S: Syscalls>(S);

impl <S: Syscalls> AirQuality<S> {
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    pub fn register_listener<'share, F: Fn(i32)>(
        listener: &'share AirQualityListener<F>,
        subscribe: Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    pub fn read_tvoc() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_TVOC, 0, 0).to_result()
    }

    pub fn read_co2() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_CO2, 0, 0).to_result()
    }

    pub fn read_data_sync(read_type: u32) -> Result<i32, ErrorCode> {
        let data_cell: Cell<Option<i32>> = Cell::new(None);
        let listener = AirQualityListener(|data_val| {
            data_cell.set(Some(data_val));
        });

        let mut unsupported: bool = false;
        scope(|subscribe| {
            if let Ok(()) = Self::register_listener(&listener, subscribe) {
                match read_type {
                    READ_CO2 => {
                        if let Ok(()) = Self::read_co2() {
                            while data_cell.get() == None {
                                S::yield_wait();
                            }
                        }
                    }
                    READ_TVOC => {
                        if let Ok(()) = Self::read_tvoc() {
                            while data_cell.get() == None {
                                S::yield_wait();
                            }
                        }
                    }
                    _ => { unsupported = true; }
                }
            }
        });

        if unsupported {
            return Err(ErrorCode::NoSupport);
        }
        match data_cell.get() {
            None => Err(ErrorCode::Busy),
            Some(data_val) => Ok(data_val),
        }
    }
}

pub struct AirQualityListener<F: Fn(i32)>(pub F);
impl<F: Fn(i32)> Upcall<OneId<DRIVER_NUM, 0>> for AirQualityListener<F> {
    fn upcall(&self, data_val: u32, _arg1: u32, _arg2: u32) {
        self.0(data_val as i32)
    }
}

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60007;

// Command IDs

const EXISTS: u32 = 0;
const READ_CO2: u32 = 2;
const READ_TVOC: u32 = 3;
