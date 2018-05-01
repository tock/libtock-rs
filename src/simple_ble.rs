use alloc::String;
use alloc::Vec;
use callback::CallbackSubscription;
use callback::SubscribableCallback;
use result::TockResult;
use shared_memory::SharedMemory;
use syscalls;

const DRIVER_NUMBER: usize = 0x30000;
pub const MAX_PAYLOAD_SIZE: usize = 9;
pub const BUFFER_SIZE_ADVERTISE: usize = 39;
pub const BUFFER_SIZE_SCAN: usize = 39;

mod ble_commands {
    pub const START_ADVERTISING: usize = 0;
    pub const SET_ADVERTISING_INTERVAL: usize = 3;
    pub const ALLOW_ADVERTISMENT_BUFFER: usize = 0x32;
    pub const REQ_ADV_ADDRESS: usize = 6;
    pub const BLE_PASSIVE_SCAN_SUB: usize = 0;
    pub const ALLOW_SCAN_BUFFER: usize = 0x31;
    pub const PASSIVE_SCAN: usize = 5;
}

mod gap_flags {
    pub const BLE_DISCOVERABLE: u8 = 0x02;
    pub const BLE_NOT_DISCOVERABLE: u8 = 0x01;
    pub const ONLY_LE: u8 = 0x04;
}

pub mod gap_data {
    pub const COMPLETE_LIST_16BIT_SERVICE_IDS: usize = 0x03;
    pub const COMPLETE_LOCAL_NAME: usize = 0x09;
    pub const SET_FLAGS: usize = 1;
    pub const SERVICE_DATA: usize = 0x16;
}

pub struct BleAdvertisingDriver;

impl BleAdvertisingDriver {
    pub fn create_advertising_buffer() -> [u8; BUFFER_SIZE_ADVERTISE] {
        [0; BUFFER_SIZE_ADVERTISE]
    }
    pub fn initialize<'a>(
        interval: u16,
        mut name: String,
        uuid: &'a mut [u8; 2],
        stay_visible: bool,
        service_payload: &'a mut Vec<u8>,
        advertising_buffer: &'a mut [u8; BUFFER_SIZE_ADVERTISE],
    ) -> TockResult<SharedMemory<'a>> {
        let mut flags: [u8; 1] = [gap_flags::ONLY_LE | (if stay_visible {
            gap_flags::BLE_DISCOVERABLE
        } else {
            gap_flags::BLE_NOT_DISCOVERABLE
        })];
        let shared_memory = syscalls::allow(
            DRIVER_NUMBER,
            ble_commands::ALLOW_ADVERTISMENT_BUFFER,
            advertising_buffer,
        )?;

        Self::set_advertising_interval(interval)?;
        Self::request_adv_address()?;
        let _name_handle = Self::set_local_name(&mut name)?;
        let _uuid_handle = Self::set_uuid(uuid)?;
        let _flags_handle = Self::set_flags(&mut flags)?;
        let _payload_handle = Self::set_service_payload(service_payload)?;
        Self::start_advertising()?;
        Ok(shared_memory)
    }

    // TODO: Write generic error converter

    fn set_advertising_interval(interval: u16) -> TockResult<()> {
        unsafe {
            syscalls::command(
                DRIVER_NUMBER,
                ble_commands::SET_ADVERTISING_INTERVAL,
                interval as usize,
                0,
            )
        }?;
        Ok(())
    }

    fn request_adv_address() -> TockResult<()> {
        unsafe { syscalls::command(DRIVER_NUMBER, ble_commands::REQ_ADV_ADDRESS, 0, 0) }?;
        Ok(())
    }

    fn set_local_name(name: &mut String) -> TockResult<SharedMemory> {
        unsafe {
            syscalls::allow(
                DRIVER_NUMBER,
                gap_data::COMPLETE_LOCAL_NAME,
                name.as_bytes_mut(),
            )
        }
    }

    fn set_uuid(uuid: &mut [u8]) -> TockResult<SharedMemory> {
        syscalls::allow(
            DRIVER_NUMBER,
            gap_data::COMPLETE_LIST_16BIT_SERVICE_IDS,
            uuid,
        )
    }

    fn set_flags(flags: &mut [u8; 1]) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, gap_data::SET_FLAGS, flags)
    }

    fn set_service_payload(service_payload: &mut [u8]) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, gap_data::SERVICE_DATA, service_payload)
    }

    fn start_advertising() -> TockResult<()> {
        unsafe { syscalls::command(DRIVER_NUMBER, ble_commands::START_ADVERTISING, 0, 0) }?;
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
    fn call_rust(&mut self, arg0: usize, arg1: usize, _: usize) {
        (self.callback)(arg0, arg1);
    }
}

pub struct BleDriver;

impl BleDriver {
    pub fn create_scan_buffer() -> [u8; BUFFER_SIZE_SCAN] {
        [0; BUFFER_SIZE_SCAN]
    }

    pub fn share_memory(scan_buffer: &mut [u8; BUFFER_SIZE_SCAN]) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, ble_commands::ALLOW_SCAN_BUFFER, scan_buffer)
    }

    pub fn start<CB>(callback: &mut BleCallback<CB>) -> TockResult<CallbackSubscription>
    where
        BleCallback<CB>: SubscribableCallback,
    {
        let subscription =
            syscalls::subscribe(DRIVER_NUMBER, ble_commands::BLE_PASSIVE_SCAN_SUB, callback)?;
        unsafe { syscalls::command(DRIVER_NUMBER, ble_commands::PASSIVE_SCAN, 1, 0) }?;

        Ok(subscription)
    }
}
