use crate::{DriverInfo, DriverShareRef};
use libtock_platform::{CommandReturn, ErrorCode};
use std::cell::Cell;

pub struct AirQuality {
    busy: Cell<bool>,
    co2_available: Cell<bool>,
    tvoc_available: Cell<bool>,
    upcall_on_read: Cell<Option<u32>>,
    upcall_on_tuple_read: Cell<Option<(u32, u32)>>,
    share_ref: DriverShareRef,
}

impl AirQuality {
    pub fn new() -> std::rc::Rc<AirQuality> {
        std::rc::Rc::new(AirQuality {
            busy: Cell::new(false),
            co2_available: Cell::new(true),
            tvoc_available: Cell::new(true),
            upcall_on_read: Cell::new(None),
            upcall_on_tuple_read: Cell::new(None),
            share_ref: Default::default(),
        })
    }

    pub fn set_co2_available(&self, co2_available: bool) {
        self.co2_available.set(co2_available);
    }

    pub fn set_tvoc_available(&self, tvoc_available: bool) {
        self.tvoc_available.set(tvoc_available);
    }

    pub fn is_busy(&self) -> bool {
        self.busy.get()
    }

    pub fn set_value(&self, value: u32) {
        if self.busy.get() {
            self.share_ref
                .schedule_upcall(0, (value, 0, 0))
                .expect("Unable to schedule upcall");
            self.busy.set(false);
        }
    }
    pub fn set_value_sync(&self, value: u32) {
        self.upcall_on_read.set(Some(value));
    }
    pub fn set_values_sync(&self, co2_value: u32, tvoc_value: u32) {
        self.upcall_on_tuple_read.set(Some((co2_value, tvoc_value)));
    }
}

impl crate::fake::SyscallDriver for AirQuality {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_id: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_id {
            EXISTS => crate::command_return::success(),
            READ_CO2 => {
                if !self.co2_available.get() {
                    return crate::command_return::failure(ErrorCode::NoSupport);
                }
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }

                self.busy.set(true);
                if let Some(val) = self.upcall_on_read.take() {
                    self.set_value(val);
                }
                if let Some((co2_val, _)) = self.upcall_on_tuple_read.get() {
                    self.set_value(co2_val);
                }

                crate::command_return::success()
            }
            READ_TVOC => {
                if !self.tvoc_available.get() {
                    return crate::command_return::failure(ErrorCode::NoSupport);
                }
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }

                self.busy.set(true);
                if let Some(val) = self.upcall_on_read.take() {
                    self.set_value(val);
                }
                if let Some((_, tvoc_val)) = self.upcall_on_tuple_read.take() {
                    self.set_value(tvoc_val);
                }

                crate::command_return::success()
            }
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
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
