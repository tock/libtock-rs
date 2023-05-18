//! Implementation done by : https://github.com/teodorobert
//! A simple libtock-rs example. Plays Ode of Joy using the buzzer.
#![no_main]
#![no_std]

use core::fmt::Write;
use core::time::Duration;
use libtock::buzzer::{Buzzer, Note};
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x800}

// Adapted from https://github.com/robsoncouto/arduino-songs

// Notes in the form of (note_frequency, note_delay in musical terms)
const MELODY: [(Note, i32); 62] = [
    (Note::E4, 4),
    (Note::E4, 4),
    (Note::F4, 4),
    (Note::G4, 4),
    (Note::G4, 4),
    (Note::F4, 4),
    (Note::E4, 4),
    (Note::D4, 4),
    (Note::C4, 4),
    (Note::C4, 4),
    (Note::D4, 4),
    (Note::E4, 4),
    (Note::E4, -4),
    (Note::D4, 8),
    (Note::D4, 2),
    (Note::E4, 4),
    (Note::E4, 4),
    (Note::F4, 4),
    (Note::G4, 4),
    (Note::G4, 4),
    (Note::F4, 4),
    (Note::E4, 4),
    (Note::D4, 4),
    (Note::C4, 4),
    (Note::C4, 4),
    (Note::D4, 4),
    (Note::E4, 4),
    (Note::D4, -4),
    (Note::C4, 8),
    (Note::C4, 2),
    (Note::D4, 4),
    (Note::D4, 4),
    (Note::E4, 4),
    (Note::C4, 4),
    (Note::D4, 4),
    (Note::E4, 8),
    (Note::F4, 8),
    (Note::E4, 4),
    (Note::C4, 4),
    (Note::D4, 4),
    (Note::E4, 8),
    (Note::F4, 8),
    (Note::E4, 4),
    (Note::D4, 4),
    (Note::C4, 4),
    (Note::D4, 4),
    (Note::G3, 2),
    (Note::E4, 4),
    (Note::E4, 4),
    (Note::F4, 4),
    (Note::G4, 4),
    (Note::G4, 4),
    (Note::F4, 4),
    (Note::E4, 4),
    (Note::D4, 4),
    (Note::C4, 4),
    (Note::C4, 4),
    (Note::D4, 4),
    (Note::E4, 4),
    (Note::D4, -4),
    (Note::C4, 8),
    (Note::C4, 2),
];

const TEMPO: u32 = 114;
const WHOLE_NOTE: u32 = (60000 * 4) / TEMPO;

fn main() {
    if let Err(_) = Buzzer::exists() {
        writeln!(Console::writer(), "There is no available buzzer").unwrap();
        return;
    }

    writeln!(Console::writer(), "Ode to Joy").unwrap();

    for (frequency, duration) in MELODY.iter() {
        let mut note_duration: Duration =
            Duration::from_millis((WHOLE_NOTE / duration.unsigned_abs()) as u64);
        // let mut note_duration = WHOLE_NOTE / duration.unsigned_abs();
        if duration < &0 {
            note_duration = note_duration * 15 / 10;
        }

        let note_duration = note_duration * 9 / 10;
        Buzzer::tone_sync(*frequency as u32 * 3, note_duration).unwrap();
    }
}
