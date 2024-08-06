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
    let driver = fake::Ipc::new(&[]);
    kernel.add_driver(&driver);

    assert_eq!(Ipc::exists(), Ok(()));
}

#[test]
fn discover_valid_service() {
    let kernel = fake::Kernel::new();
    let driver = fake::Ipc::new(&[
        fake::Process::new(b"org.tockos.test.app_0", 311149534),
        fake::Process::new(b"org.tockos.test.app_4", 202834883),
        fake::Process::new(b"org.tockos.test.app_18", 256614857),
    ]);
    kernel.add_driver(&driver);

    assert_eq!(Ipc::discover(b"org.tockos.test.app_4"), Ok(1))
}

#[test]
fn discover_invalid_service() {
    let kernel = fake::Kernel::new();
    let driver = fake::Ipc::new(&[
        fake::Process::new(b"org.tockos.test.app_0", 311149534),
        fake::Process::new(b"org.tockos.test.app_4", 202834883),
        fake::Process::new(b"org.tockos.test.app_18", 256614857),
    ]);
    kernel.add_driver(&driver);

    assert_eq!(
        Ipc::discover(b"com.test.service.app_0"),
        Err(ErrorCode::Invalid)
    )
}
