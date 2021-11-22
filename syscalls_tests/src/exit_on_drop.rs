use libtock_platform::exit_on_drop::ExitOnDrop;
use libtock_unittest::fake;

// Unwinds if `unwind` is true, otherwise just returns.
fn maybe_unwind(unwind: bool) {
    if unwind {
        panic!("Triggering stack unwinding.");
    }
}

#[cfg(not(miri))]
#[test]
fn exit() {
    let exit = libtock_unittest::exit_test("exit_on_drop::exit", || {
        let exit_on_drop: ExitOnDrop<fake::Syscalls> = Default::default();
        maybe_unwind(true);
        core::mem::forget(exit_on_drop);
    });
    assert_eq!(exit, libtock_unittest::ExitCall::Terminate(0));
}

#[test]
fn no_exit() {
    let exit_on_drop: ExitOnDrop<fake::Syscalls> = Default::default();
    maybe_unwind(false);
    core::mem::forget(exit_on_drop);
}
