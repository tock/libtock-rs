use crate::{RoAllowBuffer, RwAllowBuffer};
use libtock_platform::{CommandReturn, ErrorCode};

/// The `fake::Driver` trait is implemented by fake versions of Tock's kernel
/// APIs. It is used by `fake::Kernel` to route system calls to the fake kernel
/// APIs.
pub trait Driver: 'static {
    /// Returns this driver's ID. Used by `fake::Kernel` to route syscalls to
    /// the correct `fake::Driver` instance.
    fn id(&self) -> u32;

    // -------------------------------------------------------------------------
    // Subscribe
    // -------------------------------------------------------------------------

    /// Like the real Tock kernel, `fake::Kernel` implements Subscribe for
    /// drivers. Drivers must implement `num_upcalls` to tell `fake::Kernel` how
    /// many upcalls to store. `fake::Kernel` will reject Subscribe calls for
    /// any subscribe_number >= num_upcalls.
    fn num_upcalls(&self) -> u32;

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    /// Process a Command system call. Fake drivers should use the methods in
    /// `libtock_unittest::command_return` to construct the return value.
    fn command(&self, command_id: u32, argument0: u32, argument1: u32) -> CommandReturn;

    // -------------------------------------------------------------------------
    // Allow
    // -------------------------------------------------------------------------

    /// Process a Read-Only Allow call. Because not all Driver implementations
    /// need to support Read-Only Allow, a default implementation is provided
    /// that rejects all Read-Only Allow calls.
    fn allow_readonly(
        &self,
        buffer_number: u32,
        buffer: RoAllowBuffer,
    ) -> Result<RoAllowBuffer, (RoAllowBuffer, ErrorCode)> {
        let _ = buffer_number; // Silences the unused variable warning.
        Err((buffer, ErrorCode::NoSupport))
    }

    /// Process a Read-Write Allow call. Because not all Driver implementations
    /// need to support Read-Write Allow, a default implementation is provided
    /// that rejects all Read-Write Allow calls.
    fn allow_readwrite(
        &self,
        buffer_number: u32,
        buffer: RwAllowBuffer,
    ) -> Result<RwAllowBuffer, (RwAllowBuffer, ErrorCode)> {
        let _ = buffer_number; // Silences the unused variable warning.
        Err((buffer, ErrorCode::NoSupport))
    }
}
