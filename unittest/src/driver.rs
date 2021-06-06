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

    // TODO: Add a Subscribe API.

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    // TODO: Add a Command API.

    // -------------------------------------------------------------------------
    // Allow
    // -------------------------------------------------------------------------

    // TODO: Add an Allow API.
}
