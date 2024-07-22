use libtock_platform::{syscall_class, yield_id, RawSyscalls, Register};
use std::convert::TryInto;

unsafe impl RawSyscalls for crate::fake::Syscalls {
    unsafe fn yield1([r0]: [Register; 1]) {
        crate::fake::syscalls::assert_valid(r0);
        match r0.try_into().expect("too-large Yield ID passed") {
            yield_id::NO_WAIT => panic!("yield-no-wait called without an argument"),
            yield_id::WAIT => super::yield_impl::yield_wait(),
            id => panic!("unknown yield ID {}", id),
        }
    }

    unsafe fn yield2([r0, r1]: [Register; 2]) {
        crate::fake::syscalls::assert_valid((r0, r1));
        match r0.try_into().expect("too-large Yield ID passed") {
            yield_id::NO_WAIT => unsafe { super::yield_impl::yield_no_wait(r1.into()) },
            yield_id::WAIT => {
                // Technically it is acceptable to call yield_wait with an
                // argument, but it shouldn't be done because it's wasteful so
                // we fail the test case regardless.
                panic!("yield-wait called with an argument");
            }
            id => panic!("unknown yield ID {}", id),
        }
    }

    unsafe fn syscall1<const CLASS: usize>([r0]: [Register; 1]) -> [Register; 2] {
        match CLASS {
            syscall_class::MEMOP => super::memop_impl::memop(r0, 0u32.into()),
            _ => panic!("Unknown syscall1 call. Class: {}", CLASS),
        }
    }

    unsafe fn syscall2<const CLASS: usize>([r0, r1]: [Register; 2]) -> [Register; 2] {
        crate::fake::syscalls::assert_valid((r0, r1));
        match CLASS {
            syscall_class::MEMOP => super::memop_impl::memop(r0, r1),
            syscall_class::EXIT => super::exit_impl::exit(r0, r1),
            _ => panic!("Unknown syscall2 call. Class: {}", CLASS),
        }
    }

    unsafe fn syscall4<const CLASS: usize>([r0, r1, r2, r3]: [Register; 4]) -> [Register; 4] {
        crate::fake::syscalls::assert_valid((r0, r1, r2, r3));
        match CLASS {
            syscall_class::SUBSCRIBE => unsafe { super::subscribe_impl::subscribe(r0, r1, r2, r3) },
            syscall_class::COMMAND => super::command_impl::command(r0, r1, r2, r3),
            syscall_class::ALLOW_RW => unsafe { super::allow_rw_impl::allow_rw(r0, r1, r2, r3) },
            syscall_class::ALLOW_RO => unsafe { super::allow_ro_impl::allow_ro(r0, r1, r2, r3) },
            _ => panic!("Unknown syscall4 call. Class: {}", CLASS),
        }
    }
}
