#![no_std]

extern crate tock;

use core::fmt::Write;
use tock::console::Console;
use tock::ipc;
use tock::ipc::ble_ess::{self, ReadingType};
use tock::sensors::*;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut console = Console::new();
    let mut shared_buffer = ipc::reserve_shared_buffer();
    writeln!(console, "Starting BLE ESS").unwrap();

    let mut ess = match ble_ess::connect(&mut shared_buffer) {
        Ok(ess) => ess,
        _ => {
            writeln!(console, "BLE IPC Service not installed").unwrap();
            return;
        }
    };
    writeln!(console, "Found BLE IPC Service").unwrap();

    let mut humidity = HumiditySensor;
    let mut temperature = TemperatureSensor;
    let mut light = AmbientLightSensor;
    loop {
        // Temperature
        let temp = temperature.read();
        writeln!(console, "Temperature: {}", temp).unwrap();
        if let Err(_) = ess.set_reading(ReadingType::Temperature, temp) {
            writeln!(console, "Failed to set temperature").unwrap_or(());
        }

        // Light
        let lx = light.read();
        writeln!(console, "Light:       {}", lx).unwrap();
        if let Err(_) = ess.set_reading(ReadingType::Light, lx) {
            writeln!(console, "Failed to set temperature").unwrap_or(());
        }

        // Humidity
        let humid = humidity.read();
        writeln!(console, "Humidity:    {}", humid).unwrap();
        if let Err(_) = ess.set_reading(ReadingType::Humidity, humid) {
            writeln!(console, "Failed to set temperature").unwrap_or(());
        }

        timer::sleep(Duration::from_ms(5000))
    }
}
