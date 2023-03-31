//! A simple libtock-rs example. Plays Ode of Joy using the buzzer.
#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::buzzer::{note, Buzzer};
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x200}

// Adapted from https://github.com/robsoncouto/arduino-songs

// Notes in the form of (note_frequency, note_delay in musical terms)
const MELODY: [(u32, i32); 62] = [
    (note::E4, 4),
    (note::E4, 4),
    (note::F4, 4),
    (note::G4, 4),
    (note::G4, 4),
    (note::F4, 4),
    (note::E4, 4),
    (note::D4, 4),
    (note::C4, 4),
    (note::C4, 4),
    (note::D4, 4),
    (note::E4, 4),
    (note::E4, -4),
    (note::D4, 8),
    (note::D4, 2),
    (note::E4, 4),
    (note::E4, 4),
    (note::F4, 4),
    (note::G4, 4),
    (note::G4, 4),
    (note::F4, 4),
    (note::E4, 4),
    (note::D4, 4),
    (note::C4, 4),
    (note::C4, 4),
    (note::D4, 4),
    (note::E4, 4),
    (note::D4, -4),
    (note::C4, 8),
    (note::C4, 2),
    (note::D4, 4),
    (note::D4, 4),
    (note::E4, 4),
    (note::C4, 4),
    (note::D4, 4),
    (note::E4, 8),
    (note::F4, 8),
    (note::E4, 4),
    (note::C4, 4),
    (note::D4, 4),
    (note::E4, 8),
    (note::F4, 8),
    (note::E4, 4),
    (note::D4, 4),
    (note::C4, 4),
    (note::D4, 4),
    (note::G3, 2),
    (note::E4, 4),
    (note::E4, 4),
    (note::F4, 4),
    (note::G4, 4),
    (note::G4, 4),
    (note::F4, 4),
    (note::E4, 4),
    (note::D4, 4),
    (note::C4, 4),
    (note::C4, 4),
    (note::D4, 4),
    (note::E4, 4),
    (note::D4, -4),
    (note::C4, 8),
    (note::C4, 2),
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
        let mut note_duration = WHOLE_NOTE / duration.unsigned_abs();
        if duration < &0 {
            note_duration = note_duration * 15 / 10;
        }

        let note_duration = note_duration * 9 / 10;
        Buzzer::tone_sync(frequency * 3, note_duration).unwrap();
    }
}
