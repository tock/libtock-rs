use crate::callback::CallbackSubscription;
use crate::callback::Consumer;
use crate::result::TockResult;
use crate::syscalls;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;
use libtock_core::shared_memory::SharedMemory;

const DRIVER_NUMBER: usize = 0x20003;

pub const BUFFER_SIZE: usize = 64;

mod command_nr {
    pub const CHECK_PRESENT: usize = 0;
    pub const WRITE: usize = 1;
    pub const READ: usize = 2;
    pub const WRITE_READ: usize = 3;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 1;
}

mod allow_nr {
    pub const BUFFER: usize = 1;
}

#[non_exhaustive]
pub struct I2cDriverFactory;

impl I2cDriverFactory {
    pub fn init_driver(&mut self) -> TockResult<I2cDriver> {
        let i2c = I2cDriver {
            lifetime: PhantomData,
        };
        Ok(i2c)
    }
}

struct I2cEventConsumer;

impl<CB: FnMut(usize, usize)> Consumer<CB> for I2cEventConsumer {
    fn consume(callback: &mut CB, _: usize, _: usize, _: usize) {
        callback(0, 0);
    }
}

pub struct I2cBuffer {
    buffer: [u8; BUFFER_SIZE],
}

impl Default for I2cBuffer {
    fn default() -> Self {
        I2cBuffer {
            buffer: [0; BUFFER_SIZE],
        }
    }
}

impl Deref for I2cBuffer {
    type Target = [u8; BUFFER_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for I2cBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct I2cDriver<'a> {
    lifetime: PhantomData<&'a ()>,
}

impl<'a> I2cDriver<'a> {
    pub fn init_buffer(&self, buffer: &'a mut I2cBuffer) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::BUFFER, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn subscribe<CB: FnMut(usize, usize)>(
        &self,
        callback: &'a mut CB,
    ) -> TockResult<CallbackSubscription> {
        syscalls::subscribe::<I2cEventConsumer, _>(
            DRIVER_NUMBER,
            subscribe_nr::SUBSCRIBE_CALLBACK,
            callback,
        )
        .map_err(Into::into)
    }

    pub fn check_present(&self) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::CHECK_PRESENT, 0, 0).map_err(Into::into)
    }

    pub fn write(&self, address: usize, length: usize) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::WRITE, address, length).map_err(Into::into)
    }

    pub fn read(&self, address: usize, length: usize) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::READ, address, length).map_err(Into::into)
    }

    pub fn write_read(&self, address: usize, length: usize) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::WRITE_READ, address, length)
            .map_err(Into::into)
    }
}
