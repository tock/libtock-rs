use crate::callback::CallbackSubscription;
use crate::callback::Consumer;
use crate::result::TockResult;
use crate::syscalls;
use core::marker::PhantomData;
use libtock_core::shared_memory::SharedMemory;

const DRIVER_NUMBER: usize = 0x40003;

pub const KEY_BUFFER_SIZE: usize = 32;
pub const DATA_BUFFER_SIZE: usize = 256;
pub const DEST_BUFFER_SIZE: usize = 32;

mod command_nr {
    pub const SET_ALGORITHM: usize = 0;
    pub const RUN: usize = 1;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

mod allow_nr {
    pub const KEY: usize = 0;
    pub const DATA: usize = 1;
    pub const DEST: usize = 2;
}

#[non_exhaustive]
pub struct HmacDriverFactory;

impl HmacDriverFactory {
    pub fn init_driver(&mut self) -> TockResult<HmacDriver> {
        let hmac = HmacDriver {
            lifetime: PhantomData,
        };
        Ok(hmac)
    }
}

struct HmacEventConsumer;

impl<CB: FnMut(usize, usize)> Consumer<CB> for HmacEventConsumer {
    fn consume(callback: &mut CB, result: usize, digest: usize, _: usize) {
        callback(result, digest);
    }
}

pub struct HmacKeyBuffer {
    buffer: [u8; KEY_BUFFER_SIZE],
}

impl Default for HmacKeyBuffer {
    fn default() -> Self {
        HmacKeyBuffer {
            buffer: [0; KEY_BUFFER_SIZE],
        }
    }
}

pub struct HmacDataBuffer {
    pub buffer: [u8; DATA_BUFFER_SIZE],
}

impl Default for HmacDataBuffer {
    fn default() -> Self {
        HmacDataBuffer {
            buffer: [0; DATA_BUFFER_SIZE],
        }
    }
}

pub struct HmacDestBuffer {
    buffer: [u8; DEST_BUFFER_SIZE],
}

impl Default for HmacDestBuffer {
    fn default() -> Self {
        HmacDestBuffer {
            buffer: [0; DEST_BUFFER_SIZE],
        }
    }
}

pub struct HmacDriver<'a> {
    lifetime: PhantomData<&'a ()>,
}

impl<'a> HmacDriver<'a> {
    pub fn init_key_buffer(&self, buffer: &'a mut HmacKeyBuffer) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::KEY, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn init_data_buffer(&self, buffer: &'a mut HmacDataBuffer) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::DATA, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn init_dest_buffer(&self, buffer: &'a mut HmacDestBuffer) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::DEST, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn subscribe<CB: FnMut(usize, usize) -> () + FnMut(usize, usize)>(
        &self,
        callback: &'a mut CB,
    ) -> TockResult<CallbackSubscription> {
        syscalls::subscribe::<HmacEventConsumer, _>(
            DRIVER_NUMBER,
            subscribe_nr::SUBSCRIBE_CALLBACK,
            callback,
        )
        .map_err(Into::into)
    }

    pub fn set_algorithm(&self, hash: usize) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::SET_ALGORITHM, hash, 0)?;
        Ok(())
    }

    pub fn run(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::RUN, 0, 0)?;
        Ok(())
    }
}
