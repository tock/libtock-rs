#![no_std]

use core::cell::Cell;
use libtock_platform::{
    share, subscribe::OneId, AllowRw, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

pub struct Adc<S: Syscalls>(S);

impl<S: Syscalls> Adc<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    // Register a listener to be called when the ADC conversion is finished
    pub fn register_listener<'share, F: Fn(u32)>(
        listener: &'share ADCListener<F>,
        subscribe: share::Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Unregister the events listener
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    /// Sets the buffer in which the ADC samples will be written
    pub fn set_buffer<'share>(
        buffer: &'share mut [u8],
        allow_rw: share::Handle<AllowRw<'share, S, {DRIVER_NUM}, 0>>,
    ) -> Result<(), ErrorCode> {
        S::allow_rw::<DefaultConfig, {DRIVER_NUM}, 0>(allow_rw, buffer)
    }


    /// Initiates a synchronous ADC conversion
    /// Returns the converted ADC value or an error
    pub fn sample() -> Result<u32, ErrorCode> {
        let samp: Cell<Option<u32>> = Cell::new(None);
        let listener = ADCListener(|adc_val| {
            samp.set(Some(adc_val));
        });
        share::scope::<(AllowRw<_, {DRIVER_NUM}, 0>, Subscribe<_, {DRIVER_NUM}, 0>), _, _>(|handle| {
            let (allow_rw, subscribe) = handle.split();

            if let Ok(()) = Self::register_listener(&listener, subscribe) {
                if let Ok(()) = Self::set_buffer(&mut [], allow_rw) {
                    if let Ok(()) = S::command(DRIVER_NUM, SINGLE_SAMPLE, 0, 0).to_result() {
                        while samp.get() == None {
                            S::yield_wait();
                        }
                    }
                }
            }
        });

        match samp.get() {
            None => Err(ErrorCode::Fail),
            Some(adc_val) => Ok(adc_val),
        }
    }

    // pub fn sample_continuous()
    // pub fn sample_buffer()
    // pub fn sample_buffer_continuous()
    // pub fn stop_sampling()

    /// Returns the number of ADC resolution bits
    pub fn get_resolution_bits() -> Result<u32, ErrorCode> {
        let mut res_bits: u32 = 0;
        if let Ok(()) = S::command(DRIVER_NUM, GET_RES_BITS, 0, &mut res_bits).to_result() {
            Ok(res_bits as u32)
        } else {
            Err(ErrorCode::Fail)
        }
    }

    /// Returns the reference voltage in millivolts (mV)
    pub fn get_reference_voltage_mv() -> Result<u32, ErrorCode> {
        let mut ref_voltage: u32 = 0;
        if let Ok(()) = S::command(DRIVER_NUM, GET_VOLTAGE_REF, 0, &mut ref_voltage).to_result() {
            Ok(ref_voltage as u32)
        } else {
            Err(ErrorCode::Fail)
        }
    }

}

pub struct ADCListener<F: Fn(u32)>(pub F);

impl<F: Fn(u32)> Upcall<OneId<DRIVER_NUM, 0>> for ADCListener<F> {
    fn upcall(&self, adc_val: u32, state: u32, _arg2: u32) {
        self.0(adc_val as u32)
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
const SINGLE_SAMPLE: u32 = 1;
const REPEAT_SINGLE_SAMPLE: u32 = 2;
const MULTIPLE_SAMPLE: u32 = 3;
const CONTINUOUS_BUFF_SAMPLE: u32 = 4;
const STOP_SAMPLE: u32 = 5;
const GET_RES_BITS: u32 = 101;
const GET_VOLTAGE_REF: u32 = 102;
