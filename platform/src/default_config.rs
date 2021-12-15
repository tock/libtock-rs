/// A general purpose syscall configuration, which drivers should use as their
/// default syscall config.
pub struct DefaultConfig;

impl crate::allow_ro::Config for DefaultConfig {}
impl crate::allow_rw::Config for DefaultConfig {}
impl crate::subscribe::Config for DefaultConfig {}
