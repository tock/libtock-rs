#![no_std]

use core::cell::Cell;
use libtock_platform::{
    share, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

pub struct Adc<S: Syscalls>(S);

impl<S: Syscalls> Adc<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        // TODO(Tock 3.0): The "exists" command should return directly return
        // `Result<(), ErrorCode>` (i.e. with no `.and()` call), but the
        // current ADC driver in the kernel returns the number of ADC channels
        // instead of just success. This will be fixed in a future release of
        // Tock, but for now we workaround this issue.
        //
        // https://github.com/tock/tock/issues/3375
        S::command(DRIVER_NUM, EXISTS, 0, 0)
            .to_result::<u32, ErrorCode>()
            .and(Ok(()))
    }

    // Initiate a sample reading
    pub fn read_single_sample() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, SINGLE_SAMPLE, 0, 0).to_result()
    }

    // Register a listener to be called when the ADC conversion is finished
    pub fn register_listener<'share, F: Fn(u16)>(
        listener: &'share ADCListener<F>,
        subscribe: share::Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Unregister the events listener
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    /// Initiates a synchronous ADC conversion
    /// Returns the converted ADC value or an error
    pub fn read_single_sample_sync() -> Result<u16, ErrorCode> {
        let sample: Cell<Option<u16>> = Cell::new(None);
        let listener = ADCListener(|adc_val| {
            sample.set(Some(adc_val));
        });
        share::scope(|subscribe| {
            Self::register_listener(&listener, subscribe)?;
            Self::read_single_sample()?;
            while sample.get().is_none() {
                S::yield_wait();
            }

            match sample.get() {
                None => Err(ErrorCode::Busy),
                Some(adc_val) => Ok(adc_val),
            }
        })
    }

    /// Returns the number of ADC resolution bits
    pub fn get_resolution_bits() -> Result<u32, ErrorCode> {
        S::command(DRIVER_NUM, GET_RES_BITS, 0, 0).to_result()
    }

    /// Returns the reference voltage in millivolts (mV)
    pub fn get_reference_voltage_mv() -> Result<u32, ErrorCode> {
        S::command(DRIVER_NUM, GET_VOLTAGE_REF, 0, 0).to_result()
    }
}

pub struct ADCListener<F: Fn(u16)>(pub F);

impl<F: Fn(u16)> Upcall<OneId<DRIVER_NUM, 0>> for ADCListener<F> {
    fn upcall(&self, adc_val: u32, _arg1: u32, _arg2: u32) {
        self.0(adc_val as u16)
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x5;

// Command IDs

const EXISTS: u32 = 0;
const SINGLE_SAMPLE: u32 = 1;
// const REPEAT_SINGLE_SAMPLE: u32 = 2;
// const MULTIPLE_SAMPLE: u32 = 3;
// const CONTINUOUS_BUFF_SAMPLE: u32 = 4;
// const STOP_SAMPLE: u32 = 5;
const GET_RES_BITS: u32 = 101;
const GET_VOLTAGE_REF: u32 = 102;
