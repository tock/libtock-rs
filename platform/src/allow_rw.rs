use crate::share::List;
use crate::Syscalls;
use core::marker::PhantomData;

// -----------------------------------------------------------------------------
// `AllowRw` struct
// -----------------------------------------------------------------------------

/// A `share::Handle<AllowRw>` instance allows safe code to call Tock's
/// Read-Write Allow system call, by guaranteeing the buffer will be revoked
/// before 'share ends. It is intended for use with the `share::scope` function,
/// which offers a safe interface for constructing `share::Handle<AllowRw>`
/// instances.
pub struct AllowRw<'share, S: Syscalls, const DRIVER_NUM: u32, const BUFFER_NUM: u32> {
    _syscalls: PhantomData<S>,

    // Make this struct invariant with respect to the 'share lifetime.
    //
    // If AllowRw were covariant with respect to 'share, then an
    // `AllowRw<'static, ...>` could be used to share a buffer that has a
    // shorter lifetime. The capsule would still have access to the memory after
    // the buffer is deallocated and the memory re-used (e.g. if the buffer is
    // on the stack), allowing it to cause undefined behavior in the process.
    // Therefore, AllowRw cannot be covariant with respect to 'share.
    // Contravariance would not have this issue, but would still be confusing
    // and would be unexpected.
    //
    // Additionally, this makes AllowRw !Sync, which is probably desirable, as
    // Sync would allow for races between threads sharing buffers with the
    // kernel.
    _share: PhantomData<core::cell::Cell<&'share mut [u8]>>,
}

// We can't derive(Default) because S is not Default, and derive(Default)
// generates a Default implementation that requires S to be Default. Instead, we
// manually implement Default.
impl<'share, S: Syscalls, const DRIVER_NUM: u32, const BUFFER_NUM: u32> Default
    for AllowRw<'share, S, DRIVER_NUM, BUFFER_NUM>
{
    fn default() -> Self {
        Self {
            _syscalls: PhantomData,
            _share: PhantomData,
        }
    }
}

impl<'share, S: Syscalls, const DRIVER_NUM: u32, const BUFFER_NUM: u32> Drop
    for AllowRw<'share, S, DRIVER_NUM, BUFFER_NUM>
{
    fn drop(&mut self) {
        S::unallow_rw(DRIVER_NUM, BUFFER_NUM);
    }
}

impl<'share, S: Syscalls, const DRIVER_NUM: u32, const BUFFER_NUM: u32> List
    for AllowRw<'share, S, DRIVER_NUM, BUFFER_NUM>
{
}

// -----------------------------------------------------------------------------
// `Config` trait
// -----------------------------------------------------------------------------

/// `Config` configures the behavior of the Read-Write Allow system call. It
/// should generally be passed through by drivers, to allow application code to
/// configure error handling.
pub trait Config {
    /// Called if a Read-Write Allow call succeeds and returns a non-zero
    /// buffer. In some applications, this may indicate unexpected reentrance.
    /// By default, the non-zero buffer is ignored.
    fn returned_nonzero_buffer(_driver_num: u32, _buffer_num: u32) {}
}
