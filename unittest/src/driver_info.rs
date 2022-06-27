/// Information that a `fake::SyscallDriver` provides to the `fake::Kernel`
/// during registration. This may be expanded over time as new features are
/// added to Tock.
#[non_exhaustive]
pub struct DriverInfo {
    // All constructors of DriverInfo require the driver to specify
    // `driver_num`.
    pub(crate) driver_num: u32,

    /// The maximum number of subscriptions to support. The maximum subscribe
    /// number supported will be one less than `upcall_count`.
    pub upcall_count: u32,
}

impl DriverInfo {
    /// Creates a new `DriverInfo` with the given driver number. `upcall_count`
    /// will be initialized to zero.
    pub fn new(driver_num: u32) -> Self {
        Self {
            driver_num,
            upcall_count: 0,
        }
    }

    /// Sets `upcall_count` and returns `self`. Used similar to a builder.
    ///
    /// # Example
    /// ```
    /// use libtock_platform::CommandReturn;
    /// use libtock_unittest::{DriverInfo, fake};
    /// struct FooDriver;
    /// impl fake::SyscallDriver for FooDriver {
    ///     fn info(&self) -> DriverInfo {
    ///         DriverInfo::new(3).upcall_count(2)
    ///     }
    ///     fn command(&self, _: u32, _: u32, _: u32) -> CommandReturn {
    ///         unimplemented!("Example code");
    ///     }
    /// }
    /// ```
    pub fn upcall_count(mut self, upcall_count: u32) -> Self {
        self.upcall_count = upcall_count;
        self
    }
}
