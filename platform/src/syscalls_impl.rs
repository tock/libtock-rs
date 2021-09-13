//! Implements `Syscalls` for all types that implement `RawSyscalls`.

use crate::{
    exit_id, syscall_class, yield_id, CommandReturn, RawSyscalls, Syscalls, YieldNoWaitReturn,
};

impl<S: RawSyscalls> Syscalls for S {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------

    fn yield_no_wait() -> YieldNoWaitReturn {
        let mut flag = core::mem::MaybeUninit::<YieldNoWaitReturn>::uninit();

        unsafe {
            // Flag can be uninitialized here because the kernel promises to
            // only write to it, not read from it. MaybeUninit guarantees that
            // it is safe to write a YieldNoWaitReturn into it.
            Self::yield2([yield_id::NO_WAIT.into(), flag.as_mut_ptr().into()]);

            // yield-no-wait guarantees it sets (initializes) flag before
            // returning.
            flag.assume_init()
        }
    }

    fn yield_wait() {
        // Safety: yield-wait does not return a value, which satisfies yield1's
        // requirement. The yield-wait system call cannot trigger undefined
        // behavior on its own in any other way.
        unsafe {
            Self::yield1([yield_id::WAIT.into()]);
        }
    }

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    fn command(driver_id: u32, command_id: u32, argument0: u32, argument1: u32) -> CommandReturn {
        unsafe {
            // syscall4's documentation indicates it can be used to call
            // Command. The Command system call cannot trigger undefined
            // behavior on its own.
            let [r0, r1, r2, r3] = Self::syscall4::<{ syscall_class::COMMAND }>([
                driver_id.into(),
                command_id.into(),
                argument0.into(),
                argument1.into(),
            ]);

            // Because r0 and r1 are returned directly from the kernel, we are
            // guaranteed that if r0 represents a failure variant then r1 is an
            // error code.
            CommandReturn::new(r0.as_u32().into(), r1.as_u32(), r2.as_u32(), r3.as_u32())
        }
    }

    // -------------------------------------------------------------------------
    // Exit
    // -------------------------------------------------------------------------

    fn exit_terminate(exit_code: u32) -> ! {
        unsafe {
            // syscall2's documentation indicates it can be used to call Exit.
            // The exit system call cannot trigger undefined behavior on its
            // own.
            Self::syscall2::<{ syscall_class::EXIT }>([
                exit_id::TERMINATE.into(),
                exit_code.into(),
            ]);
            // TRD104 indicates that exit-terminate MUST always succeed and so
            // never return.
            core::hint::unreachable_unchecked()
        }
    }

    fn exit_restart(exit_code: u32) -> ! {
        unsafe {
            // syscall2's documentation indicates it can be used to call Exit.
            // The exit system call cannot trigger undefined behavior on its
            // own.
            Self::syscall2::<{ syscall_class::EXIT }>([exit_id::RESTART.into(), exit_code.into()]);
            // TRD104 indicates that exit-restart MUST always succeed and so
            // never return.
            core::hint::unreachable_unchecked()
        }
    }
}
