#![feature(asm,alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::boxed::Box;
use alloc::String;
use alloc::fmt::Write;
use tock::console::Console;
use tock::ipc;
use tock::sensors::Ninedof;

#[derive(Copy, Clone)]
struct SensorUpdate {
    sensor_type: u32,
    value: i32
}

fn setup_ipc() -> Result<(ipc::Client, Box<[u8]>), ()> {
    let mut ipc_client = ipc::Client::new(
                String::from("org.tockos.services.ble-ess"))?;
    let shared_vec = ipc_client.share(5)?;
    Ok((ipc_client, shared_vec))
}

fn main() {
    let mut console = Console::new();
    let mut ninedof = unsafe { Ninedof::new() };
    let (mut ipc_client, mut updates) = match setup_ipc() {
        Ok((c, s)) => (c, s),
        _ => {
            write!(&mut console, "Bad news...\n").unwrap();
            return;
        }
    };

    loop {
        //let accel = ninedof.read_acceleration();
        updates[4] = 0xad;
        if let Err(_) = ipc_client.notify() {
            write!(&mut console, "Nope...\n").unwrap_or(());
            return;
        }
        tock::timer::delay_ms(500);
    }
}

