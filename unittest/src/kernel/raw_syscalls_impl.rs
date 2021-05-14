use libtock_platform::{syscall_class, RawSyscalls, Register};

unsafe impl RawSyscalls for super::Kernel {
    unsafe fn yield1([Register(_r0)]: [Register; 1]) {
        // TODO: Add Yield.
    }

    unsafe fn yield2([Register(_r0), Register(_r1)]: [Register; 2]) {
        // TODO: Add Yield.
    }

    unsafe fn syscall1<const CLASS: usize>([Register(_r0)]: [Register; 1]) -> [Register; 2] {
        match CLASS {
            syscall_class::MEMOP => unimplemented!("TODO: Add Memop"),
            _ => panic!("Unknown syscall1 call. Class: {}", CLASS),
        }
    }

    unsafe fn syscall2<const CLASS: usize>(
        [Register(_r0), Register(_r1)]: [Register; 2],
    ) -> [Register; 2] {
        match CLASS {
            syscall_class::MEMOP => unimplemented!("TODO: Add Memop"),
            syscall_class::EXIT => unimplemented!("TODO: Add Exit"),
            _ => panic!("Unknown syscall2 call. Class: {}", CLASS),
        }
    }

    unsafe fn syscall4<const CLASS: usize>(
        [Register(_r0), Register(_r1), Register(_r2), Register(_r3)]: [Register; 4],
    ) -> [Register; 4] {
        match CLASS {
            syscall_class::SUBSCRIBE => unimplemented!("TODO: Add Subscribe"),
            syscall_class::COMMAND => unimplemented!("TODO: Add Command"),
            syscall_class::RW_ALLOW => unimplemented!("TODO: Add Allow"),
            syscall_class::RO_ALLOW => unimplemented!("TODO: Add Allow"),
            _ => panic!("Unknown syscall4 call. Class: {}", CLASS),
        }
    }
}

// Miri does not always check that values are valid (see `doc/MiriTips.md` in
// the root of this repository). This function uses a hack to verify a value is
// valid. If the value is invalid, Miri will detect undefined behavior when it
// executes this.
#[allow(unused)] // TODO: Remove when a system call is implemented.
pub(crate) fn assert_valid<T: core::fmt::Debug>(_value: T) {
    #[cfg(miri)]
    format!("{:?}", _value);
}
