#![no_std]

use libtock::ble_parser;
use libtock::result::TockResult;
use libtock::simple_ble;
use serde::Deserialize;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x800] = [0; 0x800];

#[derive(Deserialize)]
struct LedCommand {
    pub nr: u8,
    pub st: bool,
}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;
    let leds_driver = drivers.leds.init_driver()?;

    let mut ble_scanning_driver_factory = drivers.ble_scanning;
    let mut ble_scanning_driver = ble_scanning_driver_factory.create_driver();
    let mut ble_scanning_driver_sharing = ble_scanning_driver.share_memory()?;
    let ble_scanning_driver_scanning = ble_scanning_driver_sharing.start()?;

    loop {
        let value = ble_scanning_driver_scanning.stream_values().await;
        ble_parser::find(&value, simple_ble::gap_data::SERVICE_DATA as u8)
            .and_then(|service_data| ble_parser::extract_for_service([91, 79], service_data))
            .and_then(|payload| corepack::from_bytes::<LedCommand>(&payload).ok())
            .and_then(|msg| leds_driver.get(msg.nr as usize).ok())
            .and_then(|led| led.on().ok());
    }
}
