//! Implementation started by : https://github.com/teodorobert
//! Continued and modified by : https://github.com/SheepSeb
#![no_std]

use core::cell::Cell;
use core::time::Duration;

use libtock_platform::{
    share, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};
pub struct Buzzer<S: Syscalls>(S);

impl<S: Syscalls> Buzzer<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    /// Initiate a tone
    pub fn tone(freq: u32, duration: Duration) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, BUZZER_ON, freq, duration.as_millis() as u32).to_result()
    }

    /// Register an events listener
    pub fn register_listener<'share, F: Fn(u32)>(
        listener: &'share BuzzerListener<F>,
        subscribe: share::Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Unregister the events listener
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    /// Initiate a synchronous tone
    /// Returns Ok() if the operation was successful
    pub fn tone_sync(freq: u32, duration: Duration) -> Result<(), ErrorCode> {
        let buzzer_cell: Cell<Option<u32>> = Cell::new(None);
        let listener = BuzzerListener(|buzzer_val| {
            buzzer_cell.set(Some(buzzer_val));
        });
        share::scope(|subscribe| {
            Self::register_listener(&listener, subscribe)?;
            Self::tone(freq, duration)?;
            while buzzer_cell.get().is_none() {
                S::yield_wait();
            }
            match buzzer_cell.get() {
                None => Err(ErrorCode::Fail),
                Some(_) => Ok(()),
            }
        })
    }
}

pub struct BuzzerListener<F: Fn(u32)>(pub F);
impl<F: Fn(u32)> Upcall<OneId<DRIVER_NUM, 0>> for BuzzerListener<F> {
    fn upcall(&self, _arg0: u32, _arg1: u32, _arg2: u32) {
        (self.0)(_arg0);
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x90000;

// Command IDs
const EXISTS: u32 = 0;
const BUZZER_ON: u32 = 1;

/// The notes that can be played by the buzzer
#[allow(unused)]
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Note {
    B0 = 31,
    C1 = 33,
    CS1 = 35,
    D1 = 37,
    DS1 = 39,
    E1 = 41,
    F1 = 44,
    FS1 = 46,
    G1 = 49,
    GS1 = 52,
    A1 = 55,
    AS1 = 58,
    B1 = 62,
    C2 = 65,
    CS2 = 69,
    D2 = 73,
    DS2 = 78,
    E2 = 82,
    F2 = 87,
    FS2 = 93,
    G2 = 98,
    GS2 = 104,
    A2 = 110,
    AS2 = 117,
    B2 = 123,
    C3 = 131,
    CS3 = 139,
    D3 = 147,
    DS3 = 156,
    E3 = 165,
    F3 = 175,
    FS3 = 185,
    G3 = 196,
    GS3 = 208,
    A3 = 220,
    AS3 = 233,
    B3 = 247,
    C4 = 262,
    CS4 = 277,
    D4 = 294,
    DS4 = 311,
    E4 = 330,
    F4 = 349,
    FS4 = 370,
    G4 = 392,
    GS4 = 415,
    A4 = 440,
    AS4 = 466,
    B4 = 494,
    C5 = 523,
    CS5 = 554,
    D5 = 587,
    DS5 = 622,
    E5 = 659,
    F5 = 698,
    FS5 = 740,
    G5 = 784,
    GS5 = 831,
    A5 = 880,
    AS5 = 932,
    B5 = 988,
    C6 = 1047,
    CS6 = 1109,
    D6 = 1175,
    DS6 = 1245,
    E6 = 1319,
    F6 = 1397,
    FS6 = 1480,
    G6 = 1568,
    GS6 = 1661,
    A6 = 1760,
    AS6 = 1865,
    B6 = 1976,
    C7 = 2093,
    CS7 = 2217,
    D7 = 2349,
    DS7 = 2489,
    E7 = 2637,
    F7 = 2794,
    FS7 = 2960,
    G7 = 3136,
    GS7 = 3322,
    A7 = 3520,
    AS7 = 3729,
    B7 = 3951,
    C8 = 4186,
    CS8 = 4435,
    D8 = 4699,
    DS8 = 4978,
}
