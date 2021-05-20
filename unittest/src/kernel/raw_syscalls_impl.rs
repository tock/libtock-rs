use libtock_platform::{RawSyscalls, Register};
use std::convert::TryInto;

mod class_id {
    pub const SUBSCRIBE: usize = 1;
    pub const COMMAND: usize = 2;
    pub const RW_ALLOW: usize = 3;
    pub const RO_ALLOW: usize = 4;
    pub const MEMOP: usize = 5;
    pub const EXIT: usize = 6;
}

mod yield_id {
    pub const NO_WAIT: u32 = 0;
    pub const WAIT: u32 = 1;
}

unsafe impl RawSyscalls for super::Kernel {
    unsafe fn yield1([r0]: [Register; 1]) {
        assert_valid(r0);
        match r0.try_into().expect("too-large Yield ID passed") {
            yield_id::NO_WAIT => panic!("yield-no-wait called without an argument"),
            yield_id::WAIT => super::yield_impl::yield_wait(),
            id => panic!("unknown yield ID {}", id),
        }
    }

    unsafe fn yield2([r0, r1]: [Register; 2]) {
        assert_valid((r0, r1));
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

    unsafe fn syscall1<const CLASS: usize>([Register(_r0)]: [Register; 1]) -> [Register; 2] {
        match CLASS {
            class_id::MEMOP => unimplemented!("TODO: Add Memop"),
            _ => panic!("Unknown syscall1 call. Class: {}", CLASS),
        }
    }

    unsafe fn syscall2<const CLASS: usize>(
        [Register(_r0), Register(_r1)]: [Register; 2],
    ) -> [Register; 2] {
        match CLASS {
            class_id::MEMOP => unimplemented!("TODO: Add Memop"),
            class_id::EXIT => unimplemented!("TODO: Add Exit"),
            _ => panic!("Unknown syscall2 call. Class: {}", CLASS),
        }
    }

    unsafe fn syscall4<const CLASS: usize>(
        [Register(_r0), Register(_r1), Register(_r2), Register(_r3)]: [Register; 4],
    ) -> [Register; 4] {
        match CLASS {
            class_id::SUBSCRIBE => unimplemented!("TODO: Add Subscribe"),
            class_id::COMMAND => unimplemented!("TODO: Add Command"),
            class_id::RW_ALLOW => unimplemented!("TODO: Add Allow"),
            class_id::RO_ALLOW => unimplemented!("TODO: Add Allow"),
            _ => panic!("Unknown syscall4 call. Class: {}", CLASS),
        }
    }
}

// Miri does not always check that values are valid (see `doc/MiriTips.md` in
// the root of this repository). This function uses a hack to verify a value is
// valid. If the value is invalid, Miri will detect undefined behavior when it
// executes this.
pub(crate) fn assert_valid<T: core::fmt::Debug>(_value: T) {
    #[cfg(miri)]
    format!("{:?}", _value);
}
