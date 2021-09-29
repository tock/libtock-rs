//! Definition of the Termination trait. The main() function (set using set_main!())
//! must return a type that implements Termination.

use crate::{ErrorCode, Syscalls};

pub trait Termination {
    fn complete<S: Syscalls>(self) -> !;
}

impl Termination for () {
    fn complete<S: Syscalls>(self) -> ! {
        S::exit_terminate(0)
    }
}

impl Termination for Result<(), ErrorCode> {
    fn complete<S: Syscalls>(self) -> ! {
        let exit_code = match self {
            Ok(()) => 0,
            Err(ec) => ec as u32,
        };
        S::exit_terminate(exit_code);
    }
}
