use crate::callback::CallbackSubscription;
use crate::callback::Consumer;
use crate::result::TockResult;
use crate::syscalls;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;
use libtock_core::shared_memory::SharedMemory;

const DRIVER_NUMBER: usize = 0x20006;

pub const MASTER_BUFFER_SIZE: usize = 32;
pub const SLAVE_BUFFER_SIZE: usize = 32;

mod command_nr {
    pub const CHECK_PRESENT: usize = 0;
    pub const MASTER_WRITE: usize = 1;
    pub const MASTER_READ: usize = 2;
    pub const SLAVE_LISTEN: usize = 3;
    pub const SLAVE_ENABLE_READ: usize = 4;
    pub const SLAVE_STOP_LISTEN: usize = 5;
    pub const SLAVE_SET_ADDRESS: usize = 6;
    pub const MASTER_WRITE_READ: usize = 7;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

mod allow_nr {
    pub const MASTER_WRIE: usize = 0;
    pub const MASTER_READ: usize = 1;
    pub const SLAVE_READ: usize = 2;
    pub const SLAVE_WRITE: usize = 2;
}

#[non_exhaustive]
pub struct I2cMSDriverFactory;

impl I2cMSDriverFactory {
    pub fn init_driver(&mut self) -> TockResult<I2cMSDriver> {
        let i2c_ms = I2cMSDriver {
            lifetime: PhantomData,
        };
        Ok(i2c_ms)
    }
}

struct I2cMSEventConsumer;

impl<CB: FnMut(usize, usize)> Consumer<CB> for I2cMSEventConsumer {
    fn consume(callback: &mut CB, _: usize, _: usize, _: usize) {
        callback(0, 0);
    }
}

pub struct I2cMSMasterWriteBuffer {
    buffer: [u8; MASTER_BUFFER_SIZE],
}

impl Default for I2cMSMasterWriteBuffer {
    fn default() -> Self {
        I2cMSMasterWriteBuffer {
            buffer: [0; MASTER_BUFFER_SIZE],
        }
    }
}

impl Deref for I2cMSMasterWriteBuffer {
    type Target = [u8; MASTER_BUFFER_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for I2cMSMasterWriteBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct I2cMSMasterReadBuffer {
    buffer: [u8; MASTER_BUFFER_SIZE],
}

impl Default for I2cMSMasterReadBuffer {
    fn default() -> Self {
        I2cMSMasterReadBuffer {
            buffer: [0; MASTER_BUFFER_SIZE],
        }
    }
}

impl Deref for I2cMSMasterReadBuffer {
    type Target = [u8; MASTER_BUFFER_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for I2cMSMasterReadBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct I2cMSSlaveReadBuffer {
    buffer: [u8; SLAVE_BUFFER_SIZE],
}

impl Default for I2cMSSlaveReadBuffer {
    fn default() -> Self {
        I2cMSSlaveReadBuffer {
            buffer: [0; SLAVE_BUFFER_SIZE],
        }
    }
}

impl Deref for I2cMSSlaveReadBuffer {
    type Target = [u8; SLAVE_BUFFER_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for I2cMSSlaveReadBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct I2cMSSlaveWriteBuffer {
    buffer: [u8; SLAVE_BUFFER_SIZE],
}

impl Default for I2cMSSlaveWriteBuffer {
    fn default() -> Self {
        I2cMSSlaveWriteBuffer {
            buffer: [0; SLAVE_BUFFER_SIZE],
        }
    }
}

impl Deref for I2cMSSlaveWriteBuffer {
    type Target = [u8; SLAVE_BUFFER_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for I2cMSSlaveWriteBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct I2cMSDriver<'a> {
    lifetime: PhantomData<&'a ()>,
}

impl<'a> I2cMSDriver<'a> {
    pub fn init_master_write_buffer(
        &self,
        buffer: &'a mut I2cMSMasterWriteBuffer,
    ) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::MASTER_WRIE, &mut buffer.buffer)
            .map_err(Into::into)
    }

    pub fn init_master_read_buffer(
        &self,
        buffer: &'a mut I2cMSMasterReadBuffer,
    ) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::MASTER_READ, &mut buffer.buffer)
            .map_err(Into::into)
    }

    pub fn init_slave_read_buffer(
        &self,
        buffer: &'a mut I2cMSSlaveReadBuffer,
    ) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::SLAVE_READ, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn init_slave_write_buffer(
        &self,
        buffer: &'a mut I2cMSSlaveWriteBuffer,
    ) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::SLAVE_WRITE, &mut buffer.buffer)
            .map_err(Into::into)
    }

    pub fn subscribe<CB: FnMut(usize, usize)>(
        &self,
        callback: &'a mut CB,
    ) -> TockResult<CallbackSubscription> {
        syscalls::subscribe::<I2cMSEventConsumer, _>(
            DRIVER_NUMBER,
            subscribe_nr::SUBSCRIBE_CALLBACK,
            callback,
        )
        .map_err(Into::into)
    }

    pub fn check_present(&self) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::CHECK_PRESENT, 0, 0).map_err(Into::into)
    }

    pub fn master_write(&self, address: usize, length: usize) -> TockResult<usize> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::MASTER_WRITE,
            address & 0xFFFF | length << 16,
            0,
        )
        .map_err(Into::into)
    }

    pub fn master_read(&self, address: usize, length: usize) -> TockResult<usize> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::MASTER_READ,
            address & 0xFFFF | length << 16,
            0,
        )
        .map_err(Into::into)
    }

    pub fn slave_listen(&self) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::SLAVE_LISTEN, 0, 0).map_err(Into::into)
    }

    pub fn slave_enable_read(&self) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::SLAVE_ENABLE_READ, 0, 0).map_err(Into::into)
    }

    pub fn slave_stop_listen(&self) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::SLAVE_STOP_LISTEN, 0, 0).map_err(Into::into)
    }

    pub fn slave_set_address(&self, address: usize) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::SLAVE_SET_ADDRESS, address, 0)
            .map_err(Into::into)
    }

    pub fn master_write_read(
        &self,
        address: usize,
        read_length: usize,
        write_length: usize,
    ) -> TockResult<usize> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::MASTER_WRITE_READ,
            address & 0xFF | read_length << 8 | write_length << 16,
            0,
        )
        .map_err(Into::into)
    }
}
