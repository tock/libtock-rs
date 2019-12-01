use crate::ble_composer::BlePayload;
use crate::callback::CallbackSubscription;
use crate::callback::SubscribableCallback;
use crate::result::TockResult;
use crate::shared_memory::SharedMemory;
use crate::syscalls;

const DRIVER_NUMBER: usize = 0x30000;
pub const MAX_PAYLOAD_SIZE: usize = 9;
pub const BUFFER_SIZE_ADVERTISE: usize = 39;
pub const BUFFER_SIZE_SCAN: usize = 39;

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
pub struct BleAdvertisingDriver;

impl BleAdvertisingDriver {
    pub fn create_advertising_buffer() -> [u8; BUFFER_SIZE_ADVERTISE] {
        [0; BUFFER_SIZE_ADVERTISE]
    }
    pub fn initialize<'a>(
        &'a mut self,
        interval: usize,
        service_payload: &BlePayload,
        advertising_buffer: &'a mut [u8; BUFFER_SIZE_ADVERTISE],
    ) -> TockResult<SharedMemory<'a>> {
        let mut shared_memory = syscalls::allow(
            DRIVER_NUMBER,
            allow_nr::ALLOW_ADVERTISMENT_BUFFER,
            advertising_buffer,
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

pub struct BleCallback<CB> {
    callback: CB,
}

impl<CB> BleCallback<CB> {
    pub fn new(callback: CB) -> Self {
        BleCallback { callback }
    }
}

impl<CB: FnMut(usize, usize)> SubscribableCallback for BleCallback<CB> {
    fn call_rust(&mut self, arg1: usize, arg2: usize, _: usize) {
        (self.callback)(arg1, arg2);
    }
}

#[non_exhaustive]
pub struct BleScanningDriver;

impl BleScanningDriver {
    pub fn create_scan_buffer() -> [u8; BUFFER_SIZE_SCAN] {
        [0; BUFFER_SIZE_SCAN]
    }

    pub fn share_memory<'a, 'b>(
        &'a mut self,
        scan_buffer: &'b mut [u8; BUFFER_SIZE_SCAN],
    ) -> TockResult<SharedMemory<'b>> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::ALLOW_SCAN_BUFFER, scan_buffer).map_err(Into::into)
    }

    pub fn start<'a, CB: FnMut(usize, usize)>(
        &'a mut self,
        callback: &'a mut BleCallback<CB>,
    ) -> TockResult<CallbackSubscription> {
        let subscription =
            syscalls::subscribe_cb(DRIVER_NUMBER, subscribe_nr::BLE_PASSIVE_SCAN_SUB, callback)?;
        syscalls::command(DRIVER_NUMBER, command_nr::PASSIVE_SCAN, 1, 0)?;
        Ok(subscription)
    }
}
