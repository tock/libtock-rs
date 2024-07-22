use super::memop_impl::*;
use crate::{fake, ExpectedSyscall};
use libtock_platform::{return_variant, ErrorCode, ReturnVariant};
use std::convert::TryInto;
use std::panic::catch_unwind;

// Tests memop with expected syscalls that don't match this memop call.
#[test]
fn expected_wrong_memop() {
    let kernel = fake::Kernel::new();
    let expected_syscall = ExpectedSyscall::Memop {
        memop_num: 1,
        argument0: 1u32.into(),
        return_error: None,
    };

    kernel.add_expected_syscall(expected_syscall);
    assert!(catch_unwind(|| memop(0u32.into(), 1u32.into()))
        .expect_err("failed to catch wrong memop_num")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("expected different memop_num"));

    kernel.add_expected_syscall(expected_syscall);
    assert!(catch_unwind(|| memop(1u32.into(), 0u32.into()))
        .expect_err("failed to catch wrong memop argument0")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("expected different argument0"));
}

#[test]
fn no_kernel() {
    let result = catch_unwind(|| memop(1u32.into(), 1u32.into()));
    assert!(result
        .expect_err("failed to catch missing kernel")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("no fake::Kernel exists"));
}

#[test]
fn return_error() {
    let kernel = fake::Kernel::new();
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 1,
        argument0: 4u32.into(),
        return_error: Some(ErrorCode::NoMem),
    });
    let [r0, r1] = memop(1u32.into(), 4u32.into());
    let r0: u32 = r0.try_into().expect("too large r0");
    let r1: u32 = r1.try_into().expect("too large r1");
    let return_variant: ReturnVariant = r0.into();
    assert_eq!(return_variant, return_variant::FAILURE);
    assert_eq!(r1, ErrorCode::NoMem as u32);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn too_large_memop_num() {
    let _kernel = fake::Kernel::new();
    let result = catch_unwind(|| memop((u32::MAX as usize + 1).into(), 1u32.into()));
    assert!(result
        .expect_err("failed to catch too-large memop num")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("Too large memop num"));
}

#[test]
fn memop_using_syscall1() {
    let kernel = fake::Kernel::new();
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 2,
        argument0: 0u32.into(),
        return_error: None,
    });
    let [r0, r1] = memop(2u32.into(), 0u32.into());
    let r0: u32 = r0.try_into().expect("too large r0");
    let _r1: u32 = r1.try_into().expect("too large r1");
    let return_variant: ReturnVariant = r0.into();
    assert_eq!(return_variant, return_variant::SUCCESS);
    // No assertion for return value, could be any value from real kernel.
}
