use crate::allow_db::AllowDb;
use crate::fake;
use fake::ipc::*;
use libtock_platform::share;
use libtock_platform::{DefaultConfig, ErrorCode, Register, YieldNoWaitReturn};

const APP_0_PROCESS_ID: u32 = 311149534;
const APP_1_PROCESS_ID: u32 = 202834883;
const APP_2_PROCESS_ID: u32 = 256614857;

// Tests the command implementation.
#[test]
fn command() {
    use fake::SyscallDriver;
    let ipc = Ipc::new(&[
        Process::new(b"org.tockos.test.app_0", APP_0_PROCESS_ID),
        Process::new(b"org.tockos.test.app_1", APP_1_PROCESS_ID),
        Process::new(b"org.tockos.test.app_2", APP_2_PROCESS_ID),
    ]);

    // Exists
    assert!(ipc.command(command::EXISTS, 0, 0).is_success());

    // Discover
    let mut db: AllowDb = Default::default();

    let present_str = b"org.tockos.test.app_1";
    let present_addr: Register = (present_str as *const u8).into();
    let present_len: Register = present_str.len().into();
    let present = unsafe { db.insert_ro_buffer(present_addr, present_len) }.unwrap();
    ipc.allow_readonly(fake::ipc::allow_ro::SEARCH, present)
        .unwrap();
    assert_eq!(
        ipc.command(command::DISCOVER, 0, 0).get_success_u32(),
        Some(1)
    );

    let absent_str = b"com.example.test.app_other";
    let absent_addr: Register = (absent_str as *const u8).into();
    let absent_len: Register = absent_str.len().into();
    let absent = unsafe { db.insert_ro_buffer(absent_addr, absent_len) }.unwrap();
    ipc.allow_readonly(fake::ipc::allow_ro::SEARCH, absent)
        .unwrap();
    assert_eq!(
        ipc.command(command::DISCOVER, 0, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );

    // Service Notify
    assert_eq!(
        ipc.as_process(APP_1_PROCESS_ID, || {
            assert!(ipc.command(command::SERVICE_NOTIFY, 2, 0).is_success());
            assert_eq!(
                ipc.command(command::SERVICE_NOTIFY, 3, 0).get_failure(),
                Some(ErrorCode::Invalid)
            );
        }),
        Ok(())
    );

    // Client Notify
    assert_eq!(
        ipc.as_process(APP_2_PROCESS_ID, || {
            assert!(ipc.command(command::CLIENT_NOTIFY, 0, 0).is_success());
            assert_eq!(
                ipc.command(command::CLIENT_NOTIFY, 4, 0).get_failure(),
                Some(ErrorCode::Invalid)
            );
        }),
        Ok(())
    );
}

// Integration test that verifies Ipc works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let ipc = Ipc::new(&[
        Process::new(b"org.tockos.test.app_0", APP_0_PROCESS_ID),
        Process::new(b"org.tockos.test.app_1", APP_1_PROCESS_ID),
        Process::new(b"org.tockos.test.app_2", APP_2_PROCESS_ID),
    ]);
    kernel.add_driver(&ipc);

    // Exists
    assert!(fake::Syscalls::command(DRIVER_NUM, command::EXISTS, 0, 0).is_success());

    // Discover
    share::scope(|allow_search| {
        assert_eq!(
            fake::Syscalls::allow_ro::<DefaultConfig, DRIVER_NUM, { allow_ro::SEARCH }>(
                allow_search,
                b"org.tockos.test.app_1",
            ),
            Ok(())
        );
        assert_eq!(
            fake::Syscalls::command(DRIVER_NUM, command::DISCOVER, 0, 0).get_success_u32(),
            Some(1)
        );
    });

    share::scope(|allow_search| {
        fake::Syscalls::allow_ro::<DefaultConfig, DRIVER_NUM, { allow_ro::SEARCH }>(
            allow_search,
            b"com.example.test.app_5",
        )
        .unwrap();
        assert_eq!(
            fake::Syscalls::command(DRIVER_NUM, command::DISCOVER, 0, 0).get_failure(),
            Some(ErrorCode::Invalid)
        );
    });

    // Notify Service
    assert_eq!(
        ipc.as_process(APP_1_PROCESS_ID, || {
            let listener = Cell::<Option<(u32, u32, u32)>>::new(None);
            share::scope(|subscribe_service| {
                assert_eq!(
                    fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 2>(
                        subscribe_service,
                        &listener,
                    ),
                    Ok(())
                );
                assert!(
                    fake::Syscalls::command(DRIVER_NUM, command::SERVICE_NOTIFY, 2, 0).is_success()
                );
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
            });
            assert_eq!(listener.get(), Some((1, 0, 0)));

            assert_eq!(
                fake::Syscalls::command(DRIVER_NUM, command::SERVICE_NOTIFY, 3, 0).get_failure(),
                Some(ErrorCode::Invalid)
            );
            assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        }),
        Ok(())
    );

    // Notify Client
    assert_eq!(
        ipc.as_process(APP_2_PROCESS_ID, || {
            let listener = Cell::<Option<(u32, u32, u32)>>::new(None);
            share::scope(|subscribe_client| {
                assert_eq!(
                    fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 2>(
                        subscribe_client,
                        &listener,
                    ),
                    Ok(())
                );
                assert!(
                    fake::Syscalls::command(DRIVER_NUM, command::CLIENT_NOTIFY, 0, 0).is_success()
                );
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
            });
            assert_eq!(listener.get(), Some((2, 0, 0)));

            assert_eq!(
                fake::Syscalls::command(DRIVER_NUM, command::CLIENT_NOTIFY, 5, 0).get_failure(),
                Some(ErrorCode::Invalid)
            );
            assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        }),
        Ok(())
    );
}
