use libtock_platform::ErrorCode;
use libtock_unittest::fake;

type Servo = super::Servo<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Servo::exists(), Err(ErrorCode::Fail))
}
#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Servo::<2>::new();
    kernel.add_driver(&driver);
    assert_eq!(Servo::exists(), Ok(()));
}
#[test]
fn count() {
    let kernel = fake::Kernel::new();
    let driver = fake::Servo::<2>::new();
    kernel.add_driver(&driver);
    assert_eq!(Servo::count(), Ok(2));
}
#[test]
fn set_angle() {
    let kernel = fake::Kernel::new();
    let driver = fake::Servo::<2>::new();
    kernel.add_driver(&driver);
    assert_eq!(Servo::set_angle(1, 90), Ok(()));
}
#[test]
fn get_angle() {
    let kernel = fake::Kernel::new();
    let driver = fake::Servo::<2>::new();
    kernel.add_driver(&driver);
    assert_eq!(Servo::set_angle(1, 45), Ok(()));
    assert_eq!(Servo::get_angle(1), Ok(45));
}
