#![no_std]

use futures::future;
use libtock::ble_parser;
use libtock::result::TockResult;
use libtock::simple_ble;
use libtock::simple_ble::BleCallback;
use libtock::simple_ble::BleScanningDriver;
use serde::Deserialize;

#[derive(Deserialize)]
struct LedCommand {
    pub nr: u8,
    pub st: bool,
}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let leds_driver = drivers.leds.init_driver()?;

    let mut shared_buffer = BleScanningDriver::create_scan_buffer();
    let mut my_buffer = BleScanningDriver::create_scan_buffer();
    let shared_memory = drivers.ble_scanning.share_memory(&mut shared_buffer)?;

    let mut callback = BleCallback::new(|_: usize, _: usize| {
        shared_memory.read_bytes(&mut my_buffer[..]);
        ble_parser::find(&my_buffer, simple_ble::gap_data::SERVICE_DATA as u8)
            .and_then(|service_data| ble_parser::extract_for_service([91, 79], service_data))
            .and_then(|payload| corepack::from_bytes::<LedCommand>(&payload).ok())
            .and_then(|msg| {
                leds_driver
                    .get(msg.nr as usize)
                    .map(|led| led.set(msg.st))
                    .into()
            });
    });

    let _subscription = drivers.ble_scanning.start(&mut callback)?;

    future::pending().await
}
