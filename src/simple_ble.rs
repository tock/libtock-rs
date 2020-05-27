//! Module to provide simple ble functions as scanning and advertising

use crate::ble_composer::BlePayload;
use crate::callback::CallbackSubscription;
use crate::callback::Consumer;
use crate::result::TockError;
use crate::result::TockResult;
use crate::shared_memory::SharedMemory;
use crate::syscalls;
use core::cell::Cell;
use core::future::Future;

const DRIVER_NUMBER: usize = 0x30000;
pub const MAX_PAYLOAD_SIZE: usize = 9;
pub const BUFFER_SIZE_ADVERTISE: usize = 39;
pub(crate) const BUFFER_SIZE_SCAN: usize = 39;

mod command_nr {
    pub const START_ADVERTISING: usize = 0;
    pub const PASSIVE_SCAN: usize = 5;
}

mod subscribe_nr {
    pub const BLE_PASSIVE_SCAN_SUB: usize = 0;
}

mod allow_nr {
    pub const ALLOW_ADVERTISMENT_BUFFER: usize = 0;
    pub const ALLOW_SCAN_BUFFER: usize = 1;
}

mod gap_flags {
    pub const BLE_DISCOVERABLE: usize = 0x02;
}

pub mod gap_data {
    pub const COMPLETE_LIST_16BIT_SERVICE_IDS: usize = 0x03;
    pub const COMPLETE_LOCAL_NAME: usize = 0x09;
    pub const SET_FLAGS: usize = 1;
    pub const SERVICE_DATA: usize = 0x16;
}

#[non_exhaustive]
pub struct BleAdvertisingDriverFactory;

impl BleAdvertisingDriverFactory {
    pub fn create_driver(&mut self) -> BleAdvertisingDriver {
        // Unfortunately, there is no way to check the availability of the ble advertising driver
        BleAdvertisingDriver
    }
}

#[non_exhaustive]
pub struct BleAdvertisingDriver;

pub struct BleAdvertisingBuffer([u8; BUFFER_SIZE_ADVERTISE]);

impl AsMut<[u8]> for BleAdvertisingBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl BleAdvertisingDriver {
    pub fn create_advertising_buffer() -> BleAdvertisingBuffer {
        BleAdvertisingBuffer([0; BUFFER_SIZE_ADVERTISE])
    }
    pub fn initialize<'a, 'b>(
        &'a mut self,
        interval: usize,
        service_payload: &BlePayload,
        advertising_buffer: &'b mut BleAdvertisingBuffer,
    ) -> TockResult<SharedMemory<'b>> {
        let mut shared_memory = syscalls::allow(
            DRIVER_NUMBER,
            allow_nr::ALLOW_ADVERTISMENT_BUFFER,
            &mut advertising_buffer.0,
        )?;
        shared_memory.write_bytes(service_payload);
        Self::start_advertising(gap_flags::BLE_DISCOVERABLE, interval)?;
        Ok(shared_memory)
    }

    fn start_advertising(pdu_type: usize, interval: usize) -> TockResult<()> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::START_ADVERTISING,
            pdu_type,
            interval,
        )?;
        Ok(())
    }
}

struct BleCallback<'a> {
    read_value: &'a Cell<Option<ScanBuffer>>,
    shared_buffer: SharedMemory<'a>,
}

#[derive(Clone, Copy)]
pub struct ScanBuffer([u8; BUFFER_SIZE_SCAN]);

impl AsRef<[u8]> for ScanBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

const EMPTY_SCAN_BUFFER: ScanBuffer = ScanBuffer([0; BUFFER_SIZE_SCAN]);

#[non_exhaustive]
pub struct BleScanningDriverFactory;

impl BleScanningDriverFactory {
    pub fn create_driver(&mut self) -> BleScanningDriver {
        BleScanningDriver {
            shared_buffer: EMPTY_SCAN_BUFFER,
            read_value: Cell::new(None),
        }
    }
}

/// Uninitialized Ble Scanning Driver
///
/// Usage:
/// ```no_run
/// # use futures::stream::StreamExt;
/// # use libtock::ble_parser;
/// # use libtock::result::TockResult;
/// # use libtock::simple_ble;
/// # async fn doc() -> TockResult<()> {
/// let mut drivers = libtock::retrieve_drivers()?;
/// let led_driver = drivers.leds.init_driver()?;
/// let mut ble_scanning_driver_factory = drivers.ble_scanning;
/// let mut ble_scanning_driver = ble_scanning_driver_factory.create_driver();
/// let mut ble_scanning_driver_sharing = ble_scanning_driver.share_memory()?;
/// let ble_scanning_driver_scanning = ble_scanning_driver_sharing.start()?;
///
/// let value = ble_scanning_driver_scanning.stream_values().await;
/// # Ok(())
/// # }
/// ```
pub struct BleScanningDriver {
    shared_buffer: ScanBuffer,
    read_value: Cell<Option<ScanBuffer>>,
}

impl AsMut<[u8]> for ScanBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl BleScanningDriver {
    /// Prepare Ble Scanning Driver to share memory with the ble capsule
    pub fn share_memory(&mut self) -> TockResult<BleScanningDriverShared> {
        let shared_buffer = syscalls::allow(
            DRIVER_NUMBER,
            allow_nr::ALLOW_SCAN_BUFFER,
            &mut self.shared_buffer.0,
        )
        .map_err(Into::<TockError>::into)?;
        Ok(BleScanningDriverShared {
            read_value: &self.read_value,
            callback: BleCallback {
                read_value: &self.read_value,
                shared_buffer,
            },
        })
    }
}

/// Ble Scanning Driver in "shared buffer" state
pub struct BleScanningDriverShared<'a> {
    callback: BleCallback<'a>,
    read_value: &'a Cell<Option<ScanBuffer>>,
}

impl<'a> BleScanningDriverShared<'a> {
    /// Start scanning for ble advertising events
    pub fn start(&mut self) -> TockResult<BleScanningDriverScanning> {
        let subscription = syscalls::subscribe::<BleCallback<'a>, BleCallback<'a>>(
            DRIVER_NUMBER,
            subscribe_nr::BLE_PASSIVE_SCAN_SUB,
            &mut self.callback,
        )?;
        syscalls::command(DRIVER_NUMBER, command_nr::PASSIVE_SCAN, 1, 0)?;
        Ok(BleScanningDriverScanning {
            _subscription: subscription,
            read_value: self.read_value,
        })
    }
}

/// Ble Scanning Driver in "scanning" state
pub struct BleScanningDriverScanning<'a> {
    _subscription: CallbackSubscription<'a>,
    read_value: &'a Cell<Option<ScanBuffer>>,
}

impl<'a> BleScanningDriverScanning<'a> {
    /// Create stream of ble scanning packets
    pub fn stream_values(&'a self) -> impl Future<Output = ScanBuffer> + 'a {
        crate::futures::wait_for_value(move || {
            if let Some(temp_buffer) = self.read_value.get() {
                self.read_value.set(None);
                Some(temp_buffer)
            } else {
                None
            }
        })
    }
}

impl<'a> Consumer<Self> for BleCallback<'a> {
    fn consume(callback: &mut Self, _: usize, _: usize, _: usize) {
        let mut temporary_buffer: ScanBuffer = EMPTY_SCAN_BUFFER;

        callback.shared_buffer.read_bytes(temporary_buffer.as_mut());
        callback.read_value.set(Some(temporary_buffer));
    }
}
