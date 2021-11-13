/// A general purpose syscall configuration, which drivers should use as their
/// default syscall config.
pub struct DefaultConfig;

impl crate::subscribe::Config for DefaultConfig {}
