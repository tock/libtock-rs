#![feature(asm,alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::fmt::Write;
use tock::console::Console;
use tock::ipc::ble_ess::{self, ReadingType};
use tock::sensors::*;

fn main() {
    let mut console = Console::new();
    write!(&mut console, "Starting BLE ESS\n").unwrap();

    let mut ess = match ble_ess::connect() {
        Ok(ess) => ess,
        _ => {
            write!(&mut console, "BLE IPC Service not installed\n").unwrap();
            return
        }
    };
    write!(&mut console, "Found BLE IPC Service\n").unwrap();

    let mut humidity = HumiditySensor;
    let mut temperature = TemperatureSensor;
    let mut light = AmbientLightSensor;
    loop {

        // Temperature
        let temp = temperature.read();
        write!(&mut console, "Temperature: {}\n", temp).unwrap();
        if let Err(_) = ess.set_reading(ReadingType::Temperature, temp) {
            write!(&mut console, "Failed to set temperature\n").unwrap_or(());
        }
        
        // Light
        let lx = light.read();
        write!(&mut console, "Light:       {}\n", lx).unwrap();
        if let Err(_) = ess.set_reading(ReadingType::Light, lx) {
            write!(&mut console, "Failed to set temperature\n").unwrap_or(());
        }

        // Humidity
        let humid = humidity.read();
        write!(&mut console, "Humidity:    {}\n", humid).unwrap();
        if let Err(_) = ess.set_reading(ReadingType::Humidity, humid) {
            write!(&mut console, "Failed to set temperature\n").unwrap_or(());
        }

        tock::timer::delay_ms(5000);
    }
}

