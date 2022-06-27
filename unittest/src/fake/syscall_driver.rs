use crate::{DriverInfo, DriverShareRef, RoAllowBuffer, RwAllowBuffer};
use libtock_platform::{CommandReturn, ErrorCode};

/// The `fake::SyscallDriver` trait is implemented by fake versions of Tock's
/// kernel APIs. It is used by `fake::Kernel` to route system calls to the fake
/// kernel APIs.
pub trait SyscallDriver: 'static {
    // -------------------------------------------------------------------------
    // Functions called by `fake::Kernel` during driver registration.
    // -------------------------------------------------------------------------

    /// Returns information about this driver, including its driver number.
    fn info(&self) -> DriverInfo;

    /// Called by `fake::Kernel` to link this driver to the `fake::Kernel`.
    /// Passes a reference to data shared with the kernel (e.g. registered
    /// upcalls).
    fn register(&self, share_ref: DriverShareRef) {
        let _ = share_ref; // Silence the unused variable warning.
    }

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    /// Process a Command system call. Fake drivers should use the methods in
    /// `libtock_unittest::command_return` to construct the return value.
    fn command(&self, command_id: u32, argument0: u32, argument1: u32) -> CommandReturn;

    // -------------------------------------------------------------------------
    // Allow
    // -------------------------------------------------------------------------

    /// Process a Read-Only Allow call. Because not all `SyscallDriver`
    /// implementations need to support Read-Only Allow, a default
    /// implementation is provided that rejects all Read-Only Allow calls.
    fn allow_readonly(
        &self,
        buffer_num: u32,
        buffer: RoAllowBuffer,
    ) -> Result<RoAllowBuffer, (RoAllowBuffer, ErrorCode)> {
        let _ = buffer_num; // Silences the unused variable warning.
        Err((buffer, ErrorCode::NoSupport))
    }

    /// Process a Read-Write Allow call. Because not all SyscallDriver
    /// implementations need to support Read-Write Allow, a default
    /// implementation is provided that rejects all Read-Write Allow calls.
    fn allow_readwrite(
        &self,
        buffer_num: u32,
        buffer: RwAllowBuffer,
    ) -> Result<RwAllowBuffer, (RwAllowBuffer, ErrorCode)> {
        let _ = buffer_num; // Silences the unused variable warning.
        Err((buffer, ErrorCode::NoSupport))
    }
}
