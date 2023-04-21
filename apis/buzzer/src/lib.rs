//! Implementation started by : https://github.com/teodorobert
//! Continued and modified by : https://github.com/SheepSeb
#![no_std]

use core::cell::Cell;
use core::time::Duration;

use libtock_platform::{share, DefaultConfig, ErrorCode, Subscribe, Syscalls};
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
    pub fn register_listener<'share>(
        listener: &'share Cell<Option<(u32,)>>,
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
    pub fn tone_sync(freq: u32, duration: Duration) -> Result<u32, ErrorCode> {
        let listener = Cell::new(Some((0,)));
        share::scope(|subscribe| {
            if let Ok(()) = Self::register_listener(&listener, subscribe) {
                if let Ok(()) = Self::tone(freq, duration) {
                    while listener.get() == None {
                        S::yield_wait();
                    }
                }
            }
        });

        match listener.get() {
            None => Err(ErrorCode::Busy),
            Some(buzzer_val) => Ok(buzzer_val.0),
        }
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
pub mod note {
    pub const B0: u32 = 31;
    pub const C1: u32 = 33;
    pub const CS1: u32 = 35;
    pub const D1: u32 = 37;
    pub const DS1: u32 = 39;
    pub const E1: u32 = 41;
    pub const F1: u32 = 44;
    pub const FS1: u32 = 46;
    pub const G1: u32 = 49;
    pub const GS1: u32 = 52;
    pub const A1: u32 = 55;
    pub const AS1: u32 = 58;
    pub const B1: u32 = 62;
    pub const C2: u32 = 65;
    pub const CS2: u32 = 69;
    pub const D2: u32 = 73;
    pub const DS2: u32 = 78;
    pub const E2: u32 = 82;
    pub const F2: u32 = 87;
    pub const FS2: u32 = 93;
    pub const G2: u32 = 98;
    pub const GS2: u32 = 104;
    pub const A2: u32 = 110;
    pub const AS2: u32 = 117;
    pub const B2: u32 = 123;
    pub const C3: u32 = 131;
    pub const CS3: u32 = 139;
    pub const D3: u32 = 147;
    pub const DS3: u32 = 156;
    pub const E3: u32 = 165;
    pub const F3: u32 = 175;
    pub const FS3: u32 = 185;
    pub const G3: u32 = 196;
    pub const GS3: u32 = 208;
    pub const A3: u32 = 220;
    pub const AS3: u32 = 233;
    pub const B3: u32 = 247;
    pub const C4: u32 = 262;
    pub const CS4: u32 = 277;
    pub const D4: u32 = 294;
    pub const DS4: u32 = 311;
    pub const E4: u32 = 330;
    pub const F4: u32 = 349;
    pub const FS4: u32 = 370;
    pub const G4: u32 = 392;
    pub const GS4: u32 = 415;
    pub const A4: u32 = 440;
    pub const AS4: u32 = 466;
    pub const B4: u32 = 494;
    pub const C5: u32 = 523;
    pub const CS5: u32 = 554;
    pub const D5: u32 = 587;
    pub const DS5: u32 = 622;
    pub const E5: u32 = 659;
    pub const F5: u32 = 698;
    pub const FS5: u32 = 740;
    pub const G5: u32 = 784;
    pub const GS5: u32 = 831;
    pub const A5: u32 = 880;
    pub const AS5: u32 = 932;
    pub const B5: u32 = 988;
    pub const C6: u32 = 1047;
    pub const CS6: u32 = 1109;
    pub const D6: u32 = 1175;
    pub const DS6: u32 = 1245;
    pub const E6: u32 = 1319;
    pub const F6: u32 = 1397;
    pub const FS6: u32 = 1480;
    pub const G6: u32 = 1568;
    pub const GS6: u32 = 1661;
    pub const A6: u32 = 1760;
    pub const AS6: u32 = 1865;
    pub const B6: u32 = 1976;
    pub const C7: u32 = 2093;
    pub const CS7: u32 = 2217;
    pub const D7: u32 = 2349;
    pub const DS7: u32 = 2489;
    pub const E7: u32 = 2637;
    pub const F7: u32 = 2794;
    pub const FS7: u32 = 2960;
    pub const G7: u32 = 3136;
    pub const GS7: u32 = 3322;
    pub const A7: u32 = 3520;
    pub const AS7: u32 = 3729;
    pub const B7: u32 = 3951;
    pub const C8: u32 = 4186;
    pub const CS8: u32 = 4435;
    pub const D8: u32 = 4699;
    pub const DS8: u32 = 4978;
}
