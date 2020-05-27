use crate::callback::CallbackSubscription;
use crate::callback::Consumer;
use crate::result::TockResult;
use crate::shared_memory::SharedMemory;
use crate::syscalls;
use core::marker::PhantomData;

pub const DRIVER_NUMBER: usize = 0x0005;
pub const BUFFER_SIZE: usize = 128;

mod command_nr {
    pub const COUNT: usize = 0;
    pub const START: usize = 1;
    pub const START_REPEAT: usize = 2;
    pub const START_REPEAT_BUFFER: usize = 3;
    pub const START_REPEAT_BUFFER_ALT: usize = 4;
    pub const STOP: usize = 5;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

mod allow_nr {
    pub const BUFFER: usize = 0;
    pub const BUFFER_ALT: usize = 1;
}

#[non_exhaustive]
pub struct AdcDriverFactory;

impl AdcDriverFactory {
    pub fn init_driver(&mut self) -> TockResult<Adc> {
        let adc = Adc {
            // num_channels
            num_channels: syscalls::command(DRIVER_NUMBER, command_nr::COUNT, 0, 0)?,
            lifetime: PhantomData,
        };
        Ok(adc)
    }
}

pub struct AdcBuffer {
    // TODO: make this generic if possible with the driver impl
    buffer: [u8; BUFFER_SIZE],
}

impl AsMut<[u8]> for AdcBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

impl Default for AdcBuffer {
    fn default() -> Self {
        AdcBuffer {
            buffer: [0; BUFFER_SIZE],
        }
    }
}

pub struct Adc<'a> {
    num_channels: usize,
    lifetime: PhantomData<&'a ()>,
}

struct AdcEventConsumer;

impl<CB: FnMut(usize, usize)> Consumer<CB> for AdcEventConsumer {
    fn consume(data: &mut CB, _: usize, channel: usize, value: usize) {
        data(channel, value);
    }
}

impl<'a> Adc<'a> {
    pub fn init_buffer(&self, buffer: &'a mut AdcBuffer) -> TockResult<SharedMemory<'a>> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::BUFFER, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn init_alt_buffer(&self, alt_buffer: &'a mut AdcBuffer) -> TockResult<SharedMemory<'a>> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::BUFFER_ALT, &mut alt_buffer.buffer)
            .map_err(Into::into)
    }

    /// Return the number of available channels
    pub fn count(&self) -> usize {
        self.num_channels
    }

    pub fn subscribe<CB: FnMut(usize, usize)>(
        &self,
        callback: &'a mut CB,
    ) -> TockResult<CallbackSubscription> {
        syscalls::subscribe::<AdcEventConsumer, _>(
            DRIVER_NUMBER,
            subscribe_nr::SUBSCRIBE_CALLBACK,
            callback,
        )
        .map_err(Into::into)
    }

    /// Start a single sample of channel
    pub fn sample(&self, channel: usize) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::START, channel, 0)?;
        Ok(())
    }

    /// Start continuous sampling of channel
    pub fn sample_continuous(&self, channel: usize) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::START_REPEAT, channel, 0)?;
        Ok(())
    }

    /// Start continuous sampling to first buffer
    pub fn sample_continuous_buffered(&self, channel: usize, frequency: usize) -> TockResult<()> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::START_REPEAT_BUFFER,
            channel,
            frequency,
        )?;
        Ok(())
    }

    /// Start continuous sampling to alternating buffer
    pub fn sample_continuous_buffered_alt(
        &self,
        channel: usize,
        frequency: usize,
    ) -> TockResult<()> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::START_REPEAT_BUFFER_ALT,
            channel,
            frequency,
        )?;
        Ok(())
    }

    /// Stop any started sampling operation
    pub fn stop(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::STOP, 0, 0)?;
        Ok(())
    }
}

impl<'a> Drop for Adc<'a> {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
