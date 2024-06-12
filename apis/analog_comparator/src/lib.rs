#![no_std]

use libtock_platform::{
    share::Handle, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

pub struct AnalogComparator<S: Syscalls>(S);

impl<S: Syscalls> AnalogComparator<S> {
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    pub fn analog_comparator_comparison(channel: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, 1, channel, 0).to_result()
    }

    pub fn analog_comparator_start_comparing(channel: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, 2, channel, 0).to_result()
    }

    pub fn analog_comparator_stop_comparing(channel: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, 3, channel, 0).to_result()
    }

    pub fn analog_comparator_count() -> Result<u32, ErrorCode> {
        S::command(DRIVER_NUM, 4, 0, 0).to_result()
    }

    pub fn analog_comparator_interrupt_subscribe<'share, F: Fn(u32)>(
        listener: &'share AnalogComparatorListener<F>,
        subscribe: Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }
}

pub struct AnalogComparatorListener<F: Fn(u32)>(F);
impl<F: Fn(u32)> Upcall<OneId<DRIVER_NUM, 0>> for AnalogComparatorListener<F> {
    fn upcall(&self, arg0: u32, _arg1: u32, _arg2: u32) {
        (self.0)(arg0);
    }
}
#[cfg(test)]
mod tests;
// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------
const DRIVER_NUM: u32 = 0x7;
const EXISTS: u32 = 0;
