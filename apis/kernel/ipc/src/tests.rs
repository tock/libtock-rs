use libtock_platform::ErrorCode;
use libtock_unittest::fake;

type Ipc = super::Ipc<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Ipc::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Ipc::new();
    kernel.add_driver(&driver);

    assert_eq!(Ipc::exists(), Ok(()));
}

#[test]
fn discover_valid_service() {
    let kernel = fake::Kernel::new();
    let processes = [
        b"org.tockos.test.app_0",
        b"org.tockos.test.app_1",
        b"org.tockos.test.app_2",
    ];
    let driver = fake::Ipc::new_with_processes(&processes);
    kernel.add_driver(&driver);

    assert_eq!(Ipc::discover(b"org.tockos.test.app_1"), Ok(1))
}

#[test]
fn discover_invalid_service() {
    let kernel = fake::Kernel::new();
    let processes = [
        b"org.tockos.test.app_0",
        b"org.tockos.test.app_1",
        b"org.tockos.test.app_2",
    ];
    let driver = fake::Ipc::new_with_processes(&processes);
    kernel.add_driver(&driver);

    assert_eq!(
        Ipc::discover(b"com.test.service.app_0"),
        Err(ErrorCode::Invalid)
    )
}
