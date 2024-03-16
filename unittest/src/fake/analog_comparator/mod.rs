use crate::{DriverInfo, DriverShareRef};
use libtock_platform::{CommandReturn, ErrorCode};
use std::cell::Cell;

pub struct AnalogComparator {
    busy: Cell<bool>,
    share_ref: DriverShareRef,
}

impl AnalogComparator {
    pub fn new() -> std::rc::Rc<AnalogComparator> {
        std::rc::Rc::new(AnalogComparator {
            busy: Cell::new(false),
            share_ref: Default::default(),
        })
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
        self.set_value(value);
    }
}

impl crate::fake::SyscallDriver for AnalogComparator {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_id: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_id {
            EXISTS => crate::command_return::success(),

            1 => {
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.busy.set(true);
                crate::command_return::success()
            }
            2 => {
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.busy.set(true);
                crate::command_return::success()
            }
            3 => {
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.busy.set(true);
                crate::command_return::success()
            }
            4 => crate::command_return::success(),
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }
}

#[cfg(test)]
mod tests;

const DRIVER_NUM: u32 = 0x7;
const EXISTS: u32 = 0;
