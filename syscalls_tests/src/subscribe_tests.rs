use libtock_platform::{
    share, subscribe, CommandReturn, DefaultConfig, ErrorCode, Syscalls, YieldNoWaitReturn,
};
use libtock_unittest::{command_return, fake, DriverInfo, DriverShareRef, SyscallLogEntry};
use std::{cell::Cell, rc::Rc};

// Fake driver that accepts an upcall.
#[derive(Default)]
struct MockDriver {
    share_ref: DriverShareRef,
}

impl fake::SyscallDriver for MockDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(1).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, _: u32, _: u32, _: u32) -> CommandReturn {
        command_return::failure(ErrorCode::NoSupport)
    }
}

#[test]
fn config() {
    // Thread local used by TestConfig to indicate that returned_nonnull_upcall
    // has been called.
    std::thread_local! {static CALLED: core::cell::Cell<Option<(u32, u32)>> = const {Cell::new(None)} }
    struct TestConfig;
    impl subscribe::Config for TestConfig {
        fn returned_nonnull_upcall(driver_num: u32, subscribe_num: u32) {
            CALLED.with(|cell| cell.set(Some((driver_num, subscribe_num))));
        }
    }

    let kernel = fake::Kernel::new();
    kernel.add_driver(&std::rc::Rc::new(MockDriver::default()));
    let called = core::cell::Cell::new(false);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, TestConfig, 1, 0>(subscribe, &called),
            Ok(())
        );
        assert_eq!(CALLED.with(|cell| cell.get()), None);

        // Repeat the subscribe, which will make the kernel return the previous
        // upcall. subscribe should invoke TestConfig::returned_nonnull_upcall.
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, TestConfig, 1, 0>(subscribe, &called),
            Ok(())
        );
        assert_eq!(CALLED.with(|cell| cell.get()), Some((1, 0)));
    });
}

#[test]
fn failed() {
    let _kernel = fake::Kernel::new();
    let done = core::cell::Cell::new(false);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, 1, 2>(subscribe, &done),
            Err(ErrorCode::NoDevice)
        );
    });
}

#[test]
fn success() {
    let driver = Rc::new(MockDriver::default());
    let kernel = fake::Kernel::new();
    kernel.add_driver(&driver);
    let called = core::cell::Cell::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, 1, 0>(subscribe, &called),
            Ok(())
        );
        assert_eq!(
            kernel.take_syscall_log(),
            [SyscallLogEntry::Subscribe {
                driver_num: 1,
                subscribe_num: 0
            }]
        );
        driver.share_ref.schedule_upcall(0, (2, 3, 4)).unwrap();
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(called.get(), Some((2, 3, 4)));
        // Clear the syscall log.
        kernel.take_syscall_log();
    });
    // Verify the upcall was cleaned up correctly.
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::Subscribe {
            driver_num: 1,
            subscribe_num: 0
        }]
    );
    driver.share_ref.schedule_upcall(0, (2, 3, 4)).unwrap();
    assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
}

#[cfg(not(miri))]
#[test]
fn unwinding_upcall() {
    struct BadUpcall;

    impl libtock_platform::Upcall<subscribe::AnyId> for BadUpcall {
        fn upcall(&self, _: u32, _: u32, _: u32) {
            panic!("Beginning stack unwinding");
        }
    }

    let exit = libtock_unittest::exit_test("subscribe_tests::unwinding_upcall", || {
        let driver = Rc::new(MockDriver::default());
        let kernel = fake::Kernel::new();
        kernel.add_driver(&driver);
        let upcall = BadUpcall;
        share::scope(|subscribe| {
            assert_eq!(
                fake::Syscalls::subscribe::<_, _, DefaultConfig, 1, 0>(subscribe, &upcall),
                Ok(())
            );
            driver.share_ref.schedule_upcall(0, (2, 3, 4)).unwrap();
            assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        });
    });
    assert_eq!(exit, libtock_unittest::ExitCall::Terminate(0));
}
