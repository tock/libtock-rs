mod allow_readable;
mod allowed;
mod allowed_slice;

pub use allow_readable::AllowReadable;
pub use allowed::Allowed;
pub use allowed_slice::{AllowedSlice, OutOfBounds};

#[cfg(test)]
mod allowed_slice_tests;
#[cfg(test)]
mod allowed_tests;
