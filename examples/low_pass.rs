#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::console::Console;

use libtock::alarm::{Alarm, Milliseconds};
use libtock::runtime::{set_main, stack_size};
use libtock::sound_pressure::SoundPressure;

set_main! {main}
stack_size! {0x1000}

const SAMPLES: usize = 10;
const SAMPLE_RATE: f64 = 100.0;

fn low_pass_filter(signal: &[f64; 10], cutoff_freq: f64, sampling_rate: f64) -> [f64; SAMPLES] {
    let pi = 3.14;
    let mut filtered_signal = [0.0; SAMPLES];
    let dt = 1.0 / sampling_rate;
    let rc = 1.0 / (2.0 * pi * cutoff_freq);
    let alpha = dt / (rc + dt);
    filtered_signal[0] = signal[0];
    for i in 1..SAMPLES {
        filtered_signal[i] = alpha * signal[i] + (1.0 - alpha) * filtered_signal[i - 1];
    }
    filtered_signal
}

fn main() {
    writeln!(Console::writer(), "Sound Pressure Example\n").unwrap();
    match SoundPressure::exists() {
        Ok(()) => writeln!(Console::writer(), "sound pressure driver available").unwrap(),
        Err(_) => {
            writeln!(Console::writer(), "sound pressure driver unavailable").unwrap();
            return;
        }
    }

    let _ = SoundPressure::sound_pressure_enabled();
    writeln!(Console::writer(), "Sound Pressure Enabled:\n",).unwrap();
    let mut sound_pressure_vals = [0.0; SAMPLES];
    let mut i = 0;
    loop {
        match SoundPressure::read_pressure_sync() {
            Ok(sound_pressure_val) => {
                sound_pressure_vals[i] = sound_pressure_val as f64;
                i += 1;
                if i == SAMPLES {
                    i = 0;
                    writeln!(
                        Console::writer(),
                        "Sound Pressure: {:?}\n",
                        sound_pressure_vals
                    )
                    .unwrap();
                    writeln!(
                        Console::writer(),
                        "Filtered Signal: {:?}\n",
                        low_pass_filter(&sound_pressure_vals, 10.0, SAMPLE_RATE)
                    )
                    .unwrap();
                }
            }
            Err(_) => writeln!(Console::writer(), "error while reading sound pressure",).unwrap(),
        }
        Alarm::sleep_for(Milliseconds(SAMPLE_RATE as u32)).unwrap();
    }
}
