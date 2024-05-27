use crate::alarm::{Alarm, Milliseconds};
use crate::platform::ErrorCode;
use libtock_spi_controller as spi_controller;

pub type SpiController = spi_controller::SpiController<super::runtime::TockSyscalls>;

pub struct EmbeddedHalSpi;

impl embedded_hal::spi::ErrorType for EmbeddedHalSpi {
    type Error = ErrorCode;
}

impl embedded_hal::spi::SpiDevice for EmbeddedHalSpi {
    fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                embedded_hal::spi::Operation::Read(read_buf) => {
                    SpiController::spi_controller_read_sync(read_buf, read_buf.len() as u32)?
                }
                embedded_hal::spi::Operation::Write(write_buf) => {
                    // writeln!(Console::writer(), "Write: write_buf: {:x?}\r", write_buf).unwrap();
                    SpiController::spi_controller_write_sync(write_buf, write_buf.len() as u32)
                        .unwrap();
                }
                embedded_hal::spi::Operation::Transfer(read_buf, write_buf) => {
                    // writeln!(Console::writer(), "Transfer: write_buf: {:x?}\r", write_buf).unwrap();
                    SpiController::spi_controller_write_read_sync(
                        write_buf,
                        read_buf,
                        write_buf.len() as u32,
                    )?
                }
                embedded_hal::spi::Operation::TransferInPlace(read_write_buf) => {
                    // writeln!(Console::writer(), "TransferInPlace: read_write_buf: {:x?}\r", read_write_buf).unwrap();
                    SpiController::spi_controller_inplace_write_read_sync(
                        read_write_buf,
                        read_write_buf.len() as u32,
                    )?
                }
                embedded_hal::spi::Operation::DelayNs(time) => {
                    Alarm::sleep_for(Milliseconds(*time / 1000)).unwrap();
                }
            }
        }

        Ok(())
    }
}
