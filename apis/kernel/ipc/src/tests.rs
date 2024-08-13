use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use libtock_platform::{ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

use crate::{IpcCallData, IpcListener};

type Ipc = super::Ipc<fake::Syscalls>;

const APP_0_PROCESS_ID: u32 = 311149534;
const APP_1_PROCESS_ID: u32 = 202834883;
const APP_2_PROCESS_ID: u32 = 256614857;

const SERVICE_PROCESS_ID: u32 = 2095420182;
const CLIENT_PROCESS_ID: u32 = 969262335;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Ipc::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Ipc::new(&[]);
    kernel.add_driver(&driver);

    assert_eq!(Ipc::exists(), Ok(()));
}

// Tests the discover implementation
#[test]
fn discover() {
    let kernel = fake::Kernel::new();
    let driver = fake::Ipc::new(&[
        fake::Process::new(b"org.tockos.test.app_0", APP_0_PROCESS_ID),
        fake::Process::new(b"org.tockos.test.app_1", APP_1_PROCESS_ID),
        fake::Process::new(b"org.tockos.test.app_2", APP_2_PROCESS_ID),
    ]);
    kernel.add_driver(&driver);

    assert_eq!(Ipc::discover(b"org.tockos.test.app_1"), Ok(1));
    assert_eq!(
        Ipc::discover(b"com.example.test.app_0"),
        Err(ErrorCode::Invalid)
    )
}

// Tests the register and notify service implementations
#[test]
fn register_and_notify_service() {
    static SERVICE_NOTIFIED: AtomicBool = AtomicBool::new(false);

    fn service_callback(_data: IpcCallData) {
        SERVICE_NOTIFIED.store(true, Ordering::Relaxed);
    }

    const SERVICE_LISTENER: IpcListener<fn(IpcCallData)> = IpcListener(service_callback);

    let kernel = fake::Kernel::new();
    let driver = fake::Ipc::new(&[
        fake::Process::new(b"org.tockos.test.service", SERVICE_PROCESS_ID),
        fake::Process::new(b"org.tockos.test.client", CLIENT_PROCESS_ID),
    ]);
    kernel.add_driver(&driver);

    assert_eq!(
        driver.as_process(SERVICE_PROCESS_ID, || {
            assert_eq!(
                Ipc::register_service_listener(b"org.example.fake.service", &SERVICE_LISTENER),
                Err(ErrorCode::Invalid)
            );
            assert_eq!(
                Ipc::register_service_listener(b"org.tockos.test.service", &SERVICE_LISTENER),
                Ok(())
            );
        }),
        Ok(())
    );

    assert_eq!(
        driver.as_process(CLIENT_PROCESS_ID, || {
            assert_eq!(Ipc::notify_service(4), Err(ErrorCode::Invalid));
            assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
            assert_eq!(Ipc::notify_service(0), Ok(()));
            assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        }),
        Ok(())
    );

    assert!(SERVICE_NOTIFIED.load(Ordering::Relaxed));
}

// Tests the register and notify client implementations
#[test]
fn register_and_notify_client() {
    static CLIENT_NOTIFIED: AtomicBool = AtomicBool::new(false);

    fn client_callback(_data: IpcCallData) {
        CLIENT_NOTIFIED.store(true, Ordering::Relaxed);
    }

    const CLIENT_LISTENER: IpcListener<fn(IpcCallData)> = IpcListener(client_callback);

    let kernel = fake::Kernel::new();
    let driver = fake::Ipc::new(&[
        fake::Process::new(b"org.tockos.test.service", SERVICE_PROCESS_ID),
        fake::Process::new(b"org.tockos.test.client", CLIENT_PROCESS_ID),
    ]);
    kernel.add_driver(&driver);

    assert_eq!(
        driver.as_process(CLIENT_PROCESS_ID, || {
            assert_eq!(
                Ipc::register_client_listener(4, &CLIENT_LISTENER),
                Err(ErrorCode::Invalid)
            );
            assert_eq!(Ipc::register_client_listener(0, &CLIENT_LISTENER), Ok(()));
        }),
        Ok(())
    );

    assert_eq!(
        driver.as_process(SERVICE_PROCESS_ID, || {
            assert_eq!(Ipc::notify_client(4), Err(ErrorCode::Invalid));
            assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
            assert_eq!(Ipc::notify_client(1), Ok(()));
            assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        }),
        Ok(())
    );

    assert!(CLIENT_NOTIFIED.load(Ordering::Relaxed));
}

// Tests the share buffer implementation
//
// Note that because IPC requires casting the buffer address passed to
// upcalls from a u32 into a *mut u8, and `make test` executes Miri with
// `-Zmiri-strict-provenance`, we need to tell Miri to ignore this test.
#[test]
#[cfg_attr(miri, ignore)]
fn share() {
    static mut BUFFER: &mut [u8] = &mut [0; 16];
    static EXPECTED_ADDR: AtomicU32 = AtomicU32::new(0);
    static EXPECTED_LEN: AtomicU32 = AtomicU32::new(0);

    fn service_callback(data: IpcCallData) {
        assert_eq!(data.caller_id, 1);

        let buffer_slice = data.buffer.expect("No IPC buffer found");
        assert_eq!(
            buffer_slice.as_ptr() as u32,
            EXPECTED_ADDR.load(Ordering::Relaxed)
        );
        assert_eq!(
            buffer_slice.len() as u32,
            EXPECTED_LEN.load(Ordering::Relaxed)
        )
    }

    const SERVICE_LISTENER: IpcListener<fn(IpcCallData)> = IpcListener(service_callback);

    let kernel = fake::Kernel::new();
    let driver = fake::Ipc::new(&[
        fake::Process::new(b"org.tockos.test.service", SERVICE_PROCESS_ID),
        fake::Process::new(b"org.tockos.test.client", CLIENT_PROCESS_ID),
    ]);
    kernel.add_driver(&driver);

    assert_eq!(
        driver.as_process(SERVICE_PROCESS_ID, || {
            assert_eq!(
                Ipc::register_service_listener(b"org.tockos.test.service", &SERVICE_LISTENER),
                Ok(())
            );
        }),
        Ok(())
    );

    assert_eq!(
        driver.as_process(CLIENT_PROCESS_ID, || {
            EXPECTED_ADDR.store(unsafe { BUFFER.as_ptr() } as u32, Ordering::Relaxed);
            EXPECTED_LEN.store(unsafe { BUFFER.len() } as u32, Ordering::Relaxed);

            assert_eq!(Ipc::share(0, unsafe { BUFFER }), Ok(()));
            assert_eq!(Ipc::notify_service(4), Err(ErrorCode::Invalid));
            assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
            assert_eq!(Ipc::notify_service(0), Ok(()));
            assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        }),
        Ok(())
    );
}
