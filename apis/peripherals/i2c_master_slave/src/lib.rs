#![no_std]

use core::cell::Cell;
use libtock_platform as platform;
use libtock_platform::allow_ro::AllowRo;
use libtock_platform::allow_rw::AllowRw;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

pub struct I2CMasterSlave<S: Syscalls, C: Config = DefaultConfig>(S, C);

impl<S: Syscalls, C: Config> I2CMasterSlave<S, C> {
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, i2c_master_slave_cmd::EXISTS, 0, 0).to_result()
    }

    /// # Summary
    ///
    /// Perform an I2C write to the slave device on @addr.
    ///
    /// # Parameter
    ///
    /// * `addr`: Slave device address
    /// * `buf`: Storage buffer, this should be bigger than @len
    /// * `len`: Number of bytes to write from @buf
    ///
    /// # Returns
    /// On success: Returns Ok(()), @len bytes were written from @buf.
    /// On failure: Err(ErrorCode), with failure ErrorCode.
    pub fn i2c_master_slave_write_sync(
        addr: u16,
        buffer: &[u8],
        len: u16,
    ) -> Result<(), ErrorCode> {
        // We could write just the buffer length, but this may lead to
        // ambiguities for the caller. So Err out early.
        if len as usize > buffer.len() {
            return Err(ErrorCode::NoMem);
        }
        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        // The kernel will split this argument into upper length and lower address.
        let cmd_arg0: u32 = (len as u32) << 16 | addr as u32;
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { ro_allow::MASTER_TX }>,
                Subscribe<_, DRIVER_NUM, { subscribe::MASTER_WRITE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();

            S::allow_ro::<C, DRIVER_NUM, { i2c_buffers::MASTER_WRITE }>(allow_ro, buffer)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::MASTER_WRITE }>(subscribe, &called)?;

            S::command(DRIVER_NUM, i2c_master_slave_cmd::MASTER_WRITE, cmd_arg0, 0)
                .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, status, _r1)) = called.get() {
                    // Kernel uses a different cmd number for this...
                    assert_eq!(r0, 0);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }

    /// # Summary
    ///
    /// Perform an I2C read from the the slave device with the slave address of @addr.
    ///
    /// # Parameter
    ///
    /// * `addr`: Slave device address
    /// * `buf`: Storage buffer, this should be bigger than @len
    /// * `len`: Number of bytes to read into @buf
    ///
    /// # Returns
    /// On success: Returns Ok(()) with @bytes_received valid.
    /// On failure: Err(ErrorCode), Failure ErrorCode and @bytes_received is invalid.
    ///
    /// Note: @bytes_received is the first return tuple index (valid only on success).
    pub fn i2c_master_slave_read_sync(
        addr: u16,
        buf: &mut [u8],
        len: u16,
    ) -> (usize, Result<(), ErrorCode>) {
        if len as usize > buf.len() {
            return (0, Err(ErrorCode::NoMem));
        }
        // This is the total amount of bytes read if the operation was a success.
        // Otherwise, it is invalid.
        let mut bytes_received: usize = core::cmp::min(buf.len(), len as usize);
        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        // The kernel will split this argument into upper length and lower address.
        let cmd_arg0: u32 = (len as u32) << 16 | addr as u32;
        let r = share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { rw_allow::MASTER_RX }>,
                Subscribe<_, DRIVER_NUM, { subscribe::MASTER_READ }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { i2c_buffers::MASTER_READ }>(allow_rw, buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::MASTER_READ }>(subscribe, &called)?;
            // When this fails, `called` is guaranteed unmodified,
            // because upcalls are never processed until we call `yield`.
            S::command(DRIVER_NUM, i2c_master_slave_cmd::MASTER_READ, cmd_arg0, 0)
                .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, _read_len, status)) = called.get() {
                    // TODO: The kernel I2C api does not currently return the read_len, so this
                    // will be invalid. We should keep track, likely assume the transfer was
                    // done if no error. See: tock@capsules/core/src/i2c_master_slave_driver.rs:129
                    // see: https://github.com/tock/tock/issues/3735
                    // Kernel uses a different cmd number for this...
                    assert_eq!(r0, 1);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        });
        // If the operation failed, make bytes received zero so that the caller isn't confused in case
        // the error is not handled properly. That is, in case of an error, we cannot guarantee the
        // number of bytes received.
        if r.is_err() {
            bytes_received = 0;
        }
        (bytes_received, r)
    }

    /// # Summary
    ///
    /// Perform an I2C write followed by a read.
    ///
    /// Note: The kernel uses the TX buffer for both actions, such that if you request a
    ///       a read that exceeds the buffer length of @w_buf, the read will be
    ///       limited to the capacity of @w_buf. This API will detect such a case
    ///       and error to avoid ambiguities until we have a better solution in the kernel.
    ///
    /// # Parameter
    ///
    /// * `addr`: Slave device address
    /// * `w_buf`: Write buffer
    /// * `r_buf`: Read buffer
    /// * `w_len`: Number of bytes to write from @w_buf
    /// * `r_len`: Number of bytes to read into @r_buf
    ///
    /// # Returns
    /// On success: Returns Ok(()) with @bytes_received valid.
    /// On failure: Err(ErrorCode), Failure ErrorCode and @bytes_received is invalid.
    ///
    /// Note: @bytes_received is the first return tuple index (valid only on success).
    pub fn i2c_master_slave_write_read_sync(
        addr: u16,
        w_buf: &mut [u8],
        r_buf: &mut [u8],
        w_len: u16,
        r_len: u16,
    ) -> (usize, Result<(), ErrorCode>) {
        if w_len as usize > w_buf.len() || r_len as usize > r_buf.len() {
            return (0, Err(ErrorCode::NoMem));
        }
        // TODO: Kernel uses the TX Buffer to perform both RX/TX for a write_read, so if
        // the @w_buff is smaller than @r_len. The subsequent read will stop prematurely.
        // So let's error here until that is addressed.
        if r_len as usize > w_buf.len() {
            return (0, Err(ErrorCode::NoMem));
        }
        // This is the total amount of bytes read if the operation was a success.
        // Otherwise, it is invalid.
        let mut bytes_received: usize = core::cmp::min(r_buf.len(), r_len as usize);
        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);

        let cmd_arg0: u32 = (w_len as u32) << 16 | (r_len as u32) << 8 | addr as u32;

        let r = share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { rw_allow::MASTER_RX }>,
                AllowRo<_, DRIVER_NUM, { ro_allow::MASTER_TX }>,
                Subscribe<_, DRIVER_NUM, { subscribe::MASTER_WRITE_READ }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, allow_ro, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { i2c_buffers::MASTER_READ }>(allow_rw, r_buf)?;
            S::allow_ro::<C, DRIVER_NUM, { i2c_buffers::MASTER_WRITE }>(allow_ro, w_buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::MASTER_WRITE_READ }>(
                subscribe, &called,
            )?;
            // When this fails, `called` is guaranteed unmodified,
            // because upcalls are never processed until we call `yield`.
            S::command(
                DRIVER_NUM,
                i2c_master_slave_cmd::MASTER_WRITE_READ,
                cmd_arg0,
                0,
            )
            .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, _read_len, status)) = called.get() {
                    // TODO: The kernel I2C api does not currently return the read_len, so this
                    // will be invalid. We should keep track, likely assume the transfer was
                    // done if no error. See: tock@capsules/core/src/i2c_master_slave_driver.rs:129
                    // see: https://github.com/tock/tock/issues/3735
                    assert_eq!(r0, i2c_master_slave_cmd::MASTER_WRITE_READ);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        });
        // If the operation failed, make bytes received zero so that the caller isn't confused in case
        // the error is not handled properly. That is, in case of an error, we cannot guarantee the
        // number of bytes received.
        if r.is_err() {
            bytes_received = 0;
        }
        (bytes_received, r)
    }

    /// # Summary
    ///
    /// Set the slave address for this device for slave mode operation. The IP should respond
    /// to @addr.
    ///
    /// # Parameter
    ///
    /// * `addr`: Slave device address to set
    ///
    /// # Returns
    /// On success: Returns Ok(())
    /// On failure: Err(ErrorCode)
    pub fn i2c_master_slave_set_slave_address(addr: u8) -> Result<(), ErrorCode> {
        // We do not count the R/W bit as part of the address, so the
        // valid range is 0x00-0x7f
        if addr > 0x7f {
            return Err(ErrorCode::Invalid);
        }
        S::command(
            DRIVER_NUM,
            i2c_master_slave_cmd::SLAVE_SET_ADDR,
            addr as u32,
            0,
        )
        .to_result()
    }

    /// # Summary
    ///
    /// Expect a write from master into the buffer pointed by @buf. This function is
    /// synchronous and returns only when the operation has completed.
    ///
    /// TODO: Add async support
    ///
    /// Note: As we do not know the size of data to be sent from a master device,
    ///       it is suggested to allocated a large buffer to accommodate bigger transfers.
    ///
    /// # Parameter
    ///
    /// * `buf`: Buffer into which to copy data from master
    ///
    /// # Returns
    /// On success: Returns (bytes_read, Ok(()))
    /// On failure: (0, Err(ErrorCode))
    pub fn i2c_master_slave_write_recv_sync(buf: &mut [u8]) -> (usize, Result<(), ErrorCode>) {
        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        let mut bytes_recvd_ret: u32 = 0;
        let r = share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { rw_allow::SLAVE_RX }>,
                Subscribe<_, DRIVER_NUM, { subscribe::SLAVE_WRITE_RECV }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { rw_allow::SLAVE_RX }>(allow_rw, buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::SLAVE_READ }>(subscribe, &called)?;

            S::command(DRIVER_NUM, i2c_master_slave_cmd::SLAVE_START_LISTEN, 0, 0)
                .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, bytes_recvd, status)) = called.get() {
                    // TODO: Ensure we are returning from the correct upcall and not from an unexpected `read_expect`
                    //       Everything in this module subscribes to `0`. Which can be problematic from an async context.
                    assert_eq!(r0, i2c_master_slave_cmd::SLAVE_START_LISTEN);
                    return match status {
                        0 => {
                            bytes_recvd_ret = bytes_recvd;
                            Ok(())
                        }
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        });
        (bytes_recvd_ret as usize, r)
    }

    /// # Summary
    ///
    /// Expect a write from master into the buffer pointed by @buf. This function is
    /// synchronous and returns only when the operation has completed.
    ///
    /// TODO: Add async support
    ///
    /// # Parameter
    ///
    /// * `buf`: Buffer from which to transfer data from
    /// * `len`: max number of bytes from buffer to transfer
    ///
    /// # Returns
    /// On success: Returns (bytes_sent, Ok(()))
    /// On failure: (0, Err(ErrorCode))
    pub fn i2c_master_slave_read_send_sync(
        buf: &[u8],
        len: usize,
    ) -> (usize, Result<(), ErrorCode>) {
        if len > buf.len() {
            return (0, Err(ErrorCode::Invalid));
        }
        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        let mut bytes_sent_ret: u32 = 0;
        let r = share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { ro_allow::SLAVE_TX }>,
                Subscribe<_, DRIVER_NUM, { subscribe::SLAVE_READ_SEND }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();
            S::allow_ro::<C, DRIVER_NUM, { ro_allow::SLAVE_TX }>(allow_ro, buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::SLAVE_READ }>(subscribe, &called)?;

            S::command(
                DRIVER_NUM,
                i2c_master_slave_cmd::SLAVE_READ_SEND,
                len as u32,
                0,
            )
            .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, bytes_sent, status)) = called.get() {
                    // TODO: Ensure we are returning from the correct upcall and not from an unexpected `read_expect`
                    //       Everything in this module subscribes to `0`. Which can be problematic from an async context.
                    assert_eq!(r0, i2c_master_slave_cmd::SLAVE_READ_SEND);
                    return match status {
                        0 => {
                            bytes_sent_ret = bytes_sent;
                            Ok(())
                        }
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        });
        (bytes_sent_ret as usize, r)
    }
}

/// System call configuration trait for `I2CMaster`.
pub trait Config:
    platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config
{
}
impl<T: platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config>
    Config for T
{
}

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------
const DRIVER_NUM: u32 = 0x20006;

#[allow(unused)]
mod subscribe {
    // TODO: It seems like only 0 is supported by the i2c_master_slave capsule currently
    //       would be nice to improve this.
    pub const MASTER_WRITE: u32 = 0;
    pub const MASTER_WRITE_READ: u32 = 0;
    pub const MASTER_READ: u32 = 0;
    pub const SLAVE_READ: u32 = 0;
    pub const SLAVE_WRITE_RECV: u32 = 0;
    pub const SLAVE_READ_SEND: u32 = 0;
}

/// Ids for read-only allow buffers
#[allow(unused)]
mod ro_allow {
    pub const MASTER_TX: u32 = 0;
    pub const SLAVE_TX: u32 = 2;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 3;
}

/// Ids for read-write allow buffers
#[allow(unused)]
mod rw_allow {
    pub const MASTER_RX: u32 = 1;
    pub const SLAVE_RX: u32 = 3;
}

#[allow(unused)]
mod i2c_buffers {
    pub const MASTER_WRITE: u32 = 0;
    pub const MASTER_READ: u32 = 1;
    pub const SLAVE_READ: u32 = 2;
    pub const SLAVE_WRITE: u32 = 3;
}

#[allow(unused)]
mod i2c_master_slave_cmd {
    pub const EXISTS: u32 = 0;
    pub const MASTER_WRITE: u32 = 1;
    pub const MASTER_READ: u32 = 2;
    pub const SLAVE_START_LISTEN: u32 = 3;
    pub const SLAVE_READ_SEND: u32 = 4;
    pub const SLAVE_SET_ADDR: u32 = 6;
    pub const MASTER_WRITE_READ: u32 = 7;
}
