use libtock_unittest::fake;

type Ipc = super::Ipc<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert!(!Ipc::exists());
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Ipc::new();
    kernel.add_driver(&driver);

    assert!(Ipc::exists());
}
