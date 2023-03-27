#![no_std]

use core::cell::Cell;
use libtock_platform::{
    share::scope, share::Handle, DefaultConfig, ErrorCode, Subscribe, Syscalls,
};
use ReadType::{ReadCO2, ReadTVOC};

enum ReadType {
    ReadCO2 = READ_CO2 as isize,
    ReadTVOC = READ_TVOC as isize,
}

pub struct AirQuality<S: Syscalls>(S);

impl<S: Syscalls> AirQuality<S> {
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    pub fn register_listener<'share>(
        listener: &'share Cell<Option<(u32,)>>,
        subscribe: Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    pub fn read_co2() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_CO2, 0, 0).to_result()
    }

    pub fn read_tvoc() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_TVOC, 0, 0).to_result()
    }

    pub fn read_co2_sync() -> Result<u32, ErrorCode> {
        Self::read_data_sync(ReadCO2)
    }

    pub fn read_tvoc_sync() -> Result<u32, ErrorCode> {
        Self::read_data_sync(ReadTVOC)
    }

    fn read_data_sync(read_type: ReadType) -> Result<u32, ErrorCode> {
        let listener: Cell<Option<(u32,)>> = Cell::new(None);

        scope(|subscribe| {
            if let Ok(()) = Self::register_listener(&listener, subscribe) {
                match read_type {
                    ReadCO2 => {
                        if let Ok(()) = Self::read_co2() {
                            while listener.get() == None {
                                S::yield_wait();
                            }
                        }
                    }
                    ReadTVOC => {
                        if let Ok(()) = Self::read_tvoc() {
                            while listener.get() == None {
                                S::yield_wait();
                            }
                        }
                    }
                }
            }
        });

        match listener.get() {
            None => Err(ErrorCode::Busy),
            Some((data_val,)) => Ok(data_val),
        }
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
