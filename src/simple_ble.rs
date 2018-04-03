use alloc::String;
use alloc::Vec;
use callback::CallbackSubscription;
use callback::SubscribableCallback;
use callback::SubscribeInfo;
use result;
use shared_memory::ShareableMemory;
use shared_memory::SharedMemory;
use syscalls;

const DRIVER_NUMBER: usize = 0x30000;
pub const MAX_PAYLOAD_SIZE: usize = 9;
pub const BUFFER_SIZE: usize = 39;
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

pub struct AdvertisingBuffer {
    shared_memory: [u8; BUFFER_SIZE],
}

impl ShareableMemory for AdvertisingBuffer {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }
    fn allow_number(&self) -> usize {
        ble_commands::ALLOW_ADVERTISMENT_BUFFER
    }
    fn to_bytes(&mut self) -> &mut [u8] {
        &mut self.shared_memory
    }
}

pub struct BleAdvertisingDriver;

impl BleAdvertisingDriver {
    pub fn initialize(
        interval: u16,
        name: String,
        uuid: Vec<u16>,
        stay_visible: bool,
        service_payload: Vec<u8>,
    ) -> Result<SharedMemory<AdvertisingBuffer>, isize> {
        let flags: [u8; 1] = [
            gap_flags::ONLY_LE | (if stay_visible {
                gap_flags::BLE_DISCOVERABLE
            } else {
                gap_flags::BLE_NOT_DISCOVERABLE
            }),
        ];
        let buffer = AdvertisingBuffer {
            shared_memory: [0; BUFFER_SIZE],
        };
        let (_, shared_memory) = syscalls::allow_new(buffer);
        Self::set_advertising_interval(interval)?;
        Self::request_adv_address()?;
        Self::set_local_name(name)?;
        Self::set_uuid(uuid)?;
        Self::set_flags(flags)?;
        Self::set_service_payload(service_payload)?;
        Self::start_advertising()?;
        Ok(shared_memory)
    }

    // TODO: Write generic error converter

    fn set_advertising_interval(interval: u16) -> Result<(), isize> {
        let result = unsafe {
            syscalls::command(
                DRIVER_NUMBER,
                ble_commands::SET_ADVERTISING_INTERVAL,
                interval as usize,
                0,
            )
        };
        convert_result(result)
    }

    fn request_adv_address() -> Result<(), isize> {
        let result =
            unsafe { syscalls::command(DRIVER_NUMBER, ble_commands::REQ_ADV_ADDRESS, 0, 0) };
        convert_result(result)
    }

    fn set_local_name(name: String) -> Result<(), isize> {
        let result = unsafe {
            syscalls::allow(
                DRIVER_NUMBER,
                gap_data::COMPLETE_LOCAL_NAME,
                name.as_bytes(),
            )
        };
        convert_result(result)
    }

    fn set_uuid(uuid: Vec<u16>) -> Result<(), isize> {
        let result = unsafe {
            syscalls::allow16(
                DRIVER_NUMBER,
                gap_data::COMPLETE_LIST_16BIT_SERVICE_IDS,
                &uuid.to_vec(),
            )
        };
        convert_result(result)
    }

    fn set_flags(flags: [u8; 1]) -> Result<(), isize> {
        let result = unsafe { syscalls::allow(DRIVER_NUMBER, gap_data::SET_FLAGS, &flags) };
        convert_result(result)
    }

    fn set_service_payload(service_payload: Vec<u8>) -> Result<(), isize> {
        let result =
            unsafe { syscalls::allow(DRIVER_NUMBER, gap_data::SERVICE_DATA, &service_payload) };
        convert_result(result)
    }

    fn start_advertising() -> Result<(), isize> {
        let result =
            unsafe { syscalls::command(DRIVER_NUMBER, ble_commands::START_ADVERTISING, 0, 0) };
        convert_result(result)
    }
}

pub struct BleSubscribeInfo;

impl SubscribeInfo for BleSubscribeInfo {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }

    fn subscribe_number(&self) -> usize {
        ble_commands::BLE_PASSIVE_SCAN_SUB
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

pub struct ScanBuffer {
    shared_memory: [u8; BUFFER_SIZE_SCAN],
}

impl<'a> ShareableMemory for ScanBuffer {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }
    fn allow_number(&self) -> usize {
        ble_commands::ALLOW_SCAN_BUFFER
    }
    fn to_bytes(&mut self) -> &mut [u8] {
        &mut self.shared_memory
    }
}

pub struct BleDriver;

impl BleDriver {
    pub fn share_memory() -> Result<SharedMemory<ScanBuffer>, isize> {
        let (result, shared_memory) = syscalls::allow_new(ScanBuffer {
            shared_memory: [0; BUFFER_SIZE_SCAN],
        });
        convert_result(result)?;
        Ok(shared_memory)
    }

    pub fn start<CB: FnMut(usize, usize)>(
        callback: &mut BleCallback<CB>,
    ) -> Result<CallbackSubscription<BleSubscribeInfo>, isize> {
        let subscription = syscalls::subscribe(BleSubscribeInfo, callback)?;

        let result = unsafe { syscalls::command(DRIVER_NUMBER, ble_commands::PASSIVE_SCAN, 1, 0) };
        convert_result(result)?;
        Ok(subscription)
    }
}

fn convert_result(code: isize) -> Result<(), isize> {
    match code {
        result::SUCCESS => Ok(()),
        code => Err(code),
    }
}
