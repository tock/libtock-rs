#![feature(asm,alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::boxed::Box;
use alloc::String;
use alloc::fmt::Write;
use tock::console::Console;
use tock::ipc;
use tock::sensors::*;

fn setup_ipc() -> Result<(ipc::Client, Box<[u8]>), ()> {
    let mut ipc_client = ipc::Client::new(
                String::from("org.tockos.services.ble-ess"))?;
    let shared_vec = ipc_client.share(5)?;
    Ok((ipc_client, shared_vec))
}

fn set_ipc_slice<I: Into<i32>>(slice: &mut [u8], sensor_type: u32, sensor_data: I) {
    let sensor_data = Into::<i32>::into(sensor_data) as u32;
    slice[0..4].copy_from_slice(&[(sensor_type & 0xff) as u8,
                                    ((sensor_type >> 8) & 0xff) as u8,
                                    ((sensor_type >> 16) & 0xff) as u8,
                                    ((sensor_type >> 24) & 0xff) as u8]);
    slice[4..8].copy_from_slice(&[(sensor_data & 0xff) as u8,
                                ((sensor_data >> 8) & 0xff) as u8,
                                ((sensor_data >> 16) & 0xff) as u8,
                                ((sensor_data >> 24) & 0xff) as u8]);
}

fn main() {
    let mut console = Console::new();
    write!(&mut console, "Starting BLE ESS\n").unwrap();

    let (mut ipc_client, mut updates) = match setup_ipc() {
        Ok((c, s)) => (c, s),
        _ => {
            write!(&mut console, "BLE IPC Service not installed\n").unwrap();
            return;
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
        set_ipc_slice(&mut *updates, 0, temp);
        if let Err(_) = ipc_client.notify() {
            write!(&mut console, "Failed to send temperature\n").unwrap_or(());
        }
        
        // Light
        let lx = light.read();
        write!(&mut console, "Light:       {}\n", lx).unwrap();
        set_ipc_slice(&mut *updates, 1, lx);
        if let Err(_) = ipc_client.notify() {
            write!(&mut console, "Failed to send light\n").unwrap_or(());
        }

        // Humidity
        let humid = humidity.read();
        write!(&mut console, "Humidity:    {}\n", humid).unwrap();
        set_ipc_slice(&mut *updates, 2, humid);
        if let Err(_) = ipc_client.notify() {
            write!(&mut console, "Failed to send humidity\n").unwrap_or(());
        }

        tock::timer::delay_ms(5000);
    }
}

