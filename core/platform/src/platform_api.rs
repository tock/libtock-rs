//! PlatformApi is a trait presenting a safe interface to Tock's system API as
//! well as a deferred call mechanism. It is implemented by the Platform type in
//! this crate.
//!
//! PlatformApi exists so that code that uses Platform's API only needs one type
//! parameter, rather than the two required by Platform.

// A lifetime (for buffers) is omitted because it is extremely annoying to use
// as a generic constraint: you need drivers to have a 'k generic argument to
// pass into the PlatformApi<'k> constraint, but then you get errors about 'k
// being unused.
// TODO: Remove the lifetime parameter on Allowed and AllowedSlice.
pub trait PlatformApi: Copy {
    // Shares a value with the kernel. The value becomes a read-write shared
    // buffer between userspace and the kernel.
    fn allow<T: Copy>(
        self,
        driver: usize,
        minor: usize,
        buffer: &'static mut T,
    ) -> Result<crate::Allowed<'static, T>, crate::ErrorCode>;

    // Shares a slice with the kernel. The shared slice becomes a read-write
    // shared buffer between userspace and the kernel.
    fn allow_slice<T: Copy>(
        self,
        driver: usize,
        minor: usize,
        buffer: &'static mut [T],
    ) -> Result<crate::AllowedSlice<'static, T>, crate::ErrorCode>;

    // Instructs a driver to perform a specific action. If the action is
    // asynchronous, its completion will be signalled by calling a callback
    // registered using `subscribe`.
    fn command(
        self,
        driver: usize,
        command_number: usize,
        argument1: usize,
        argument2: usize
    ) -> crate::ReturnCode;

    // TODO: Finish PlatformApi

    // Executes a single callback. Will run a deferred callback if one is
    // available, or wait for one kernel callback if no deferred callback is
    // queued.
    fn run_callback(self);
}
