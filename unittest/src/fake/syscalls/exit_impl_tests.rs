use super::exit_impl::*;
use crate::{exit_test, ExitCall};

#[test]
fn exit_restart() {
    let exit_call = exit_test("fake::syscalls::exit_impl_tests::exit_restart", || {
        exit(libtock_platform::exit_id::RESTART.into(), 31415u32.into())
    });
    assert_eq!(exit_call, ExitCall::Restart(31415));
}

#[test]
fn exit_terminate() {
    let exit_call = exit_test("fake::syscalls::exit_impl_tests::exit_terminate", || {
        exit(libtock_platform::exit_id::TERMINATE.into(), 9265u32.into())
    });
    assert_eq!(exit_call, ExitCall::Terminate(9265));
}
