use crate::callback::CallbackSubscription;
use crate::callback::Consumer;
use crate::result::TockResult;
use crate::syscalls;
use core::marker::PhantomData;
use libtock_core::shared_memory::SharedMemory;

const DRIVER_NUMBER: usize = 0x40004;

pub const RECV_BUFFER_SIZE: usize = 64;
pub const SEND_BUFFER_SIZE: usize = 64;

mod command_nr {
    pub const SEND_DATA: usize = 0;
    pub const ALLOW_RECEIVE: usize = 1;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

mod allow_nr {
    pub const RECV: usize = 0;
    pub const SEND: usize = 1;
}

#[non_exhaustive]
pub struct CtapDriverFactory;

impl CtapDriverFactory {
    pub fn init_driver(&mut self) -> TockResult<CtapDriver> {
        let ctap = CtapDriver {
            lifetime: PhantomData,
        };
        Ok(ctap)
    }
}

struct CtapEventConsumer;

impl<CB: FnMut(usize, usize)> Consumer<CB> for CtapEventConsumer {
    fn consume(callback: &mut CB, sent: usize, _: usize, _: usize) {
        callback(sent, 0);
    }
}

pub struct CtapRecvBuffer {
    buffer: [u8; RECV_BUFFER_SIZE],
}

impl Default for CtapRecvBuffer {
    fn default() -> Self {
        CtapRecvBuffer {
            buffer: [0; RECV_BUFFER_SIZE],
        }
    }
}

pub struct CtapSendBuffer {
    pub buffer: [u8; SEND_BUFFER_SIZE],
}

impl CtapSendBuffer {
    pub fn new(buf: [u8; SEND_BUFFER_SIZE]) -> Self {
        CtapSendBuffer { buffer: buf }
    }
}

impl Default for CtapSendBuffer {
    fn default() -> Self {
        CtapSendBuffer {
            buffer: [0; SEND_BUFFER_SIZE],
        }
    }
}

pub struct CtapDriver<'a> {
    lifetime: PhantomData<&'a ()>,
}

impl<'a> CtapDriver<'a> {
    pub fn init_recv_buffer(&self, buffer: &'a mut CtapRecvBuffer) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::RECV, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn init_send_buffer(&self, buffer: &'a mut CtapSendBuffer) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::SEND, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn subscribe<CB: FnMut(usize, usize)>(
        &self,
        callback: &'a mut CB,
    ) -> TockResult<CallbackSubscription> {
        syscalls::subscribe::<CtapEventConsumer, _>(
            DRIVER_NUMBER,
            subscribe_nr::SUBSCRIBE_CALLBACK,
            callback,
        )
        .map_err(Into::into)
    }
    pub fn send_data(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::SEND_DATA, 0, 0)?;
        Ok(())
    }

    pub fn allow_receive(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::ALLOW_RECEIVE, 0, 0)?;
        Ok(())
    }
}
