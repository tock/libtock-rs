use alloc::String;
use alloc::Vec;
use callback::CallbackSubscription;
use callback::SubscribableCallback;
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

#[allow(dead_code)]
pub struct BleDeviceUninitialized<'a> {
    interval: u16,
    name: String,
    uuid: Vec<u16>,
    flags: Vec<u8>,
    buffer: [u8; BUFFER_SIZE],
    service_payload: &'a mut Vec<u8>,
}

#[allow(dead_code)]
pub struct BleDeviceInitialized<'a> {
    interval: &'a mut u16,
    name: &'a mut String,
    uuid: &'a mut Vec<u16>,
    flags: &'a mut Vec<u8>,
    buffer: &'a mut [u8; 39],
    service_payload: &'a mut Vec<u8>,
}

#[allow(dead_code)]
pub struct BleDeviceAdvertising<'a> {
    interval: &'a mut u16,
    name: &'a mut String,
    uuid: &'a mut Vec<u16>,
    flags: &'a mut Vec<u8>,
    buffer: &'a mut [u8; 39],
    service_payload: &'a mut Vec<u8>,
}

#[allow(dead_code)]
impl<'a> BleDeviceUninitialized<'a> {
    pub fn new(
        interval: u16,
        name: String,
        uuid: Vec<u16>,
        stay_visible: bool,
        service_payload: &'a mut Vec<u8>,
    ) -> BleDeviceUninitialized {
        let flags: [u8; 1] = [
            gap_flags::ONLY_LE | (if stay_visible {
                gap_flags::BLE_DISCOVERABLE
            } else {
                gap_flags::BLE_NOT_DISCOVERABLE
            }),
        ];

        BleDeviceUninitialized {
            interval: interval,
            name: name,
            uuid: uuid,
            flags: flags.to_vec(),
            buffer: [0; 39],
            service_payload: service_payload,
        }
    }
    pub fn initialize(&'a mut self) -> Result<BleDeviceInitialized<'a>, &'static str> {
        Ok(self)
            .and_then(|ble| ble.set_advertising_buffer())
            .and_then(|ble| ble.request_address())
            .and_then(|ble| ble.set_advertising_interval())
            .and_then(|ble| ble.set_advertising_flags())
            .and_then(|ble| ble.set_advertising_name())
            .and_then(|ble| ble.set_advertsing_uuid())
            .and_then(|ble| ble.set_service_payload())
            .and_then(|ble| {
                Ok(BleDeviceInitialized {
                    interval: &mut ble.interval,
                    name: &mut ble.name,
                    uuid: &mut ble.uuid,
                    flags: &mut ble.flags,
                    buffer: &mut ble.buffer,
                    service_payload: &mut ble.service_payload,
                })
            })
    }

    fn set_advertising_interval(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe {
            syscalls::command(
                DRIVER_NUMBER,
                ble_commands::SET_ADVERTISING_INTERVAL,
                self.interval as usize,
                0,
            )
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn request_address(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe { syscalls::command(DRIVER_NUMBER, ble_commands::REQ_ADV_ADDRESS, 0, 0) } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertising_name(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe {
            syscalls::allow(
                DRIVER_NUMBER,
                gap_data::COMPLETE_LOCAL_NAME,
                self.name.as_bytes(),
            )
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertsing_uuid(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe {
            syscalls::allow16(
                DRIVER_NUMBER,
                gap_data::COMPLETE_LIST_16BIT_SERVICE_IDS,
                &self.uuid,
            )
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertising_flags(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe { syscalls::allow(DRIVER_NUMBER, gap_data::SET_FLAGS, &self.flags) } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertising_buffer(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe {
            syscalls::allow(
                DRIVER_NUMBER,
                ble_commands::ALLOW_ADVERTISMENT_BUFFER,
                &self.buffer,
            )
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_service_payload(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe {
            syscalls::allow(DRIVER_NUMBER, gap_data::SERVICE_DATA, self.service_payload)
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }
}

impl<'a> BleDeviceInitialized<'a> {
    pub fn start_advertising(&mut self) -> Result<BleDeviceAdvertising, &'static str> {
        match unsafe { syscalls::command(DRIVER_NUMBER, ble_commands::START_ADVERTISING, 0, 0) } {
            0 => Ok(BleDeviceAdvertising {
                interval: &mut self.interval,
                name: &mut self.name,
                uuid: &mut self.uuid,
                flags: &mut self.flags,
                buffer: &mut self.buffer,
                service_payload: &mut self.service_payload,
            }),
            _ => Err(""),
        }
    }
}

pub struct BleCallback<CB> {
    callback: CB,
}

impl<CB: FnMut(usize, usize)> SubscribableCallback for BleCallback<CB> {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }

    fn subscribe_number(&self) -> usize {
        ble_commands::BLE_PASSIVE_SCAN_SUB
    }

    fn call_rust(&mut self, arg0: usize, arg1: usize, _: usize) {
        (self.callback)(arg0, arg1);
    }
}

pub struct BleDriver;

impl BleDriver {
    pub fn start<CB: FnMut(usize, usize)>(
        buffer: &[u8; BUFFER_SIZE_SCAN],
        callback: CB,
    ) -> Result<CallbackSubscription<BleCallback<CB>>, ()> {
        let (_, subscription) = syscalls::subscribe_new(BleCallback { callback });
        unsafe {
            syscalls::allow(DRIVER_NUMBER, ble_commands::ALLOW_SCAN_BUFFER, buffer);
            syscalls::command(DRIVER_NUMBER, ble_commands::PASSIVE_SCAN, 1, 0);
        }
        Ok(subscription)
    }
}
